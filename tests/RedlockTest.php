<?php

namespace RustPHP\Tests;

use Exception;
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

        $lock = $client->lock('test', 1013);

        self::assertEquals('test', $lock->getResource());
        self::assertEquals(1000, $lock->getValidityTime());
        self::assertEquals(1, $client->unlock($lock));
    }

    public function testItThrowsAnExceptionWhenLockingAlreadyLockedResource(): void
    {
        $this->expectException(FailedToAcquireLock::class);

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
    }

    public function testItThrowsAnExceptionWhenConnectionFails(): void
    {
        $this->expectException(Exception::class);
        $this->expectExceptionMessage('Failed to lock: Connection refused (os error 111)');

        $client = new Redlock(
            [
                'redis://127.0.0.1:1664',
                'redis://127.0.0.1:8686',
                'redis://127.0.0.1:1337',
            ],
            3,
            200
        );

        $client->lock('test', 1000);
    }
}
