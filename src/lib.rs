use ext_php_rs::builders::ClassBuilder;
use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::{ce, ClassEntry, ModuleEntry};
use ext_php_rs::{info_table_end, info_table_row, info_table_start, prelude::*};
use redis::RedisResult;
use redlock::RedLock as RedLockLib;

const UNLOCK_SCRIPT: &str = r"if redis.call('get',KEYS[1]) == ARGV[1] then
                                return redis.call('del',KEYS[1])
                              else
                                return 0
                              end";

static mut FAILED_TO_ACQUIRE_LOCK: Option<&'static ClassEntry> = None;
static mut FAILED_TO_UNLOCK: Option<&'static ClassEntry> = None;

#[php_class(name = "RustPHP\\Extension\\Redlock\\LockResource")]
#[derive(Debug)]
pub struct LockResource {
    pub resource: Vec<u8>,
    pub value: Vec<u8>,
    pub validity_time: usize,
}

#[php_impl]
impl LockResource {
    pub fn __construct(resource: String, value: String, validity_time: usize) -> Self {
        Self {
            resource: resource.as_bytes().to_vec(),
            value: value.as_bytes().to_vec(),
            validity_time,
        }
    }
}

#[php_class(name = "RustPHP\\Extension\\Redlock\\Redlock")]
#[derive(Debug)]
pub struct Redlock {
    client: RedLockLib,
}

#[php_impl(rename_methods = "camelCase")]
impl Redlock {
    /// Create a new pool of redis server to distribute the redlock algorithm.
    ///
    /// @param string[] $servers Servers array (e.g: ["redis://127.0.0.1:6380/", ...]).
    /// @param int $retryCount Retry count defaults to 3.
    /// @param int $delay Retry delay in ms, defaults to 200ms.
    ///
    /// @return Redblock
    #[optional(retry_count, delay)]
    #[defaults(retry_count = 3, delay = 200)]
    pub fn __construct(servers: Vec<String>, retry_count: Option<i32>, delay: Option<i32>) -> Self {
        let mut client = RedLockLib::new(servers);

        if let (Some(d), Some(rc)) = (delay, retry_count) {
            client.set_retry(rc as u32, d as u32);
        }

        Self { client }
    }

    /// Lock a given resource such as a string.
    ///
    /// @param string $resource
    /// @param int $ttl Requested TTL in milliseconds.
    ///
    /// @throws FailedToAcquireLock
    ///
    /// @return LockResource
    pub fn lock(&self, resource: String, ttl: usize) -> PhpResult<LockResource> {
        let lock = self.client.lock(resource.as_bytes(), ttl);

        match lock {
            Ok(l) => match l {
                Some(lo) => Ok(LockResource {
                    resource: lo.resource,
                    value: lo.val,
                    validity_time: lo.validity_time,
                }),
                None => Err(PhpException::new(
                    format!("Failed to acquire lock for resource: {}", resource),
                    0,
                    unsafe { FAILED_TO_ACQUIRE_LOCK.expect("did not set exception ce") },
                )),
            },
            Err(err) => Err(PhpException::default(format!("Failed to lock: {}", err))),
        }
    }

    pub fn unlock(&self, lock: &LockResource) -> PhpResult<i32> {
        for client in &self.client.servers {
            // we don't really care about a server down.
            let mut con = match client.get_connection() {
                Err(_) => continue,
                Ok(val) => val,
            };
            let script = redis::Script::new(UNLOCK_SCRIPT);
            let result: RedisResult<i32> =
                script.key(&lock.resource).arg(&lock.value).invoke(&mut con);
            match result {
                Ok(val) => return Ok(val),
                Err(err) => {
                    return Err(PhpException::new(
                        format!("Failed to unlock resource: {}", err),
                        0,
                        unsafe { FAILED_TO_UNLOCK.expect("did not set exception ce") },
                    ))
                }
            };
        }

        Err(PhpException::new(
            "Failed to unlock resource: no matching key on all servers".into(),
            0,
            unsafe { FAILED_TO_UNLOCK.expect("did not set exception ce") },
        ))
    }
}

#[php_startup]
pub fn startup() {
    let ce_aquire_lock =
        ClassBuilder::new("RustPHP\\Extension\\Redlock\\Exception\\FailedToAcquireLock")
            .extends(ce::exception())
            .build()
            .expect("Failed to acquire lock resource");
    unsafe { FAILED_TO_ACQUIRE_LOCK.replace(ce_aquire_lock) };
    let ce_unlock = ClassBuilder::new("RustPHP\\Extension\\Redlock\\Exception\\FailedToUnlock")
        .extends(ce::exception())
        .build()
        .expect("Failed to unlock resource");
    unsafe { FAILED_TO_UNLOCK.replace(ce_unlock) };
}

/// Used by the `phpinfo()` function and when you run `php -i`.
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("ext-redlock-php", "enabled");
    info_table_end!();
}

// Required to register the extension with PHP.
#[php_module]
pub fn phpmodule(module: ModuleBuilder) -> ModuleBuilder {
    module.info_function(php_module_info)
}
