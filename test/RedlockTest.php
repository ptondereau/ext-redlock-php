<?php

namespace RustPHP\Tests;

use PHPUnit\Framework\TestCase;
use RustPHP\Extension\Redlock\Exception\FailedToAcquireLock;
use RustPHP\Extension\Redlock\Redlock;

class RedlockTest extends TestCase
{
    public function testItLocksAResource(): void
    {
        $client = new Redlock(
            [
                'redis://127.0.0.1:6377',
                'redis://127.0.0.1:6378',
                'redis://127.0.0.1:6379',
            ],
            3,
            200
        );

        $lock = $client->lock('test', 1000);

        self::assertEquals('test', $lock->getResource());
        self::assertEquals('test', $lock->getValue());
        self::assertEquals(1000, $lock->getValidityTime());
    }

    public function testItThrowsAnExceptionWhenLockingAlreadyLockedResource(): void
    {
        $client = new Redlock(
            [
                'redis://127.0.0.1:6377',
                'redis://127.0.0.1:6378',
                'redis://127.0.0.1:6379',
            ],
            3,
            200
        );

        $client->lock('test', 1000);
        $client->lock('test', 1000);

        $this->expectException(FailedToAcquireLock::class);
    }
}
