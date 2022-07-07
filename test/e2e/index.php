<?php

$client = new \RustPHP\Extension\Redlock\Redlock(
    [
        'redis://127.0.0.1:6377',
        'redis://127.0.0.1:6378',
        'redis://127.0.0.1:6379',
    ],
    3,
    200
);

$lock = $client->lock('test', 1000);
