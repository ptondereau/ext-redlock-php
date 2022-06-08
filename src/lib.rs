use ext_php_rs::exception::PhpException;
use ext_php_rs::zend::{ce, ModuleEntry};
use ext_php_rs::{info_table_end, info_table_row, info_table_start, prelude::*};
use redis::RedisResult;
use redlock::RedLock as RedLockLib;

const UNLOCK_SCRIPT: &str = r"if redis.call('get',KEYS[1]) == ARGV[1] then
                                return redis.call('del',KEYS[1])
                              else
                                return 0
                              end";

#[php_class(name = "RustPHP\\Extension\\Redlock\\Exception\\FailedToAcquireLock")]
#[extends(ce::exception())]
pub struct FailedToAcquireLock;

#[php_class(name = "RustPHP\\Extension\\Redlock\\Exception\\FailedToUnlock")]
#[extends(ce::exception())]
pub struct FailedToUnlock;

#[php_class(name = "RustPHP\\Extension\\Redlock\\LockResource")]
#[derive(Debug)]
pub struct LockResource {
    pub resource: Vec<u8>,
    pub value: Vec<u8>,
    pub validity_time: usize,
}

#[php_impl]
impl LockResource {
    #[constructor]
    pub fn new(resource: String, value: String, validity_time: usize) -> Self {
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

#[php_impl]
impl Redlock {
    /// Create a new pool of redis server to distribute the redlock algorithm.
    ///
    /// @param string[] $servers Servers array (e.g: ["redis://127.0.0.1:6380/", ...]).
    /// @param int $retry_count Retry count defaults to 3.
    /// @param int $delay Retry delay in ms, defaults to 200ms.
    ///
    /// @return Redblock
    #[optional(retry_count, delay)]
    #[defaults(retry_count = 3, delay = 200)]
    #[constructor]
    pub fn new(servers: Vec<String>, retry_count: i32, delay: i32) -> Self {
        let mut client = RedLockLib::new(servers);
        client.set_retry(retry_count as u32, delay as u32);

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
    pub fn lock(&self, resource: String, ttl: usize) -> Result<LockResource, PhpException> {
        let lock = self.client.lock(resource.as_bytes(), ttl);

        match lock {
            Some(l) => Ok(LockResource {
                resource: l.resource,
                value: l.val,
                validity_time: l.validity_time,
            }),
            None => Err(PhpException::from_class::<FailedToAcquireLock>(format!(
                "Failed to acquire lock for resource: {}",
                resource
            ))),
        }
    }

    pub fn unlock(&self, lock: &LockResource) -> Result<i32, PhpException> {
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
                    return Err(PhpException::from_class::<FailedToUnlock>(format!(
                        "Failed to unlock resource: {}",
                        err
                    )))
                }
            };
        }

        Err(PhpException::from_class::<FailedToUnlock>(
            "Failed to unlock resource: no matching key on all servers".to_string(),
        ))
    }
}

/// Used by the `phpinfo()` function and when you run `php -i`.
pub extern "C" fn php_module_info(_module: *mut ModuleEntry) {
    info_table_start!();
    info_table_row!("ext-redlock-rs", "enabled");
    info_table_end!();
}

// Required to register the extension with PHP.
#[php_module]
pub fn phpmodule(module: ModuleBuilder) -> ModuleBuilder {
    module.info_function(php_module_info)
}
