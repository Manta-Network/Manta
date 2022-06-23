Functional Tests for Manta
==========================

Get data from `mantaPay_pull` rpc methods:
```bash
yarn ts-node manta_pay_rpc.ts --address=some_address
```
The `address` is optional, with default `ws://127.0.0.1:9801`

Current tests:

1. `check_single_rpc_performance` starting from 0 indices, which should not take more than ~200ms
1. `check_full_sync_order_and_performance` which should be ordered correctly