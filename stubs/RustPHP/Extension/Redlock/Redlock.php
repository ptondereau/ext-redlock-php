<?php
/**
 * auto generated file by PHPExtensionStubGenerator
 */
namespace RustPHP\Extension\Redlock;

/**
 * auto generated file by PHPExtensionStubGenerator
 */
class Redlock
{

    public function lock(string $resource, int $ttl) : \RustPHP\Extension\Redlock\LockResource
    {
    }

    public function unlock(\RustPHP\Extension\Redlock\LockResource $lock) : int
    {
    }

    public function __construct(array $servers, ?int $retry_count = 3, ?int $delay = 200)
    {
    }


}
