# node1
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "node1_babe_mnemonics", "0xa6da86747dce627b0a0cf4189ce35247a5c0c9a69570f2b5b72241beb711a141"],"id":1 }' localhost:9933
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "node1_grandpa_mnemonics", "0xc5189d7881d966d8355c403a8b490267e1ca28b471d948f1a054f536fef0ecdc"],"id":1 }' localhost:9933

# node2
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "node2_babe_mnemonics", "0xc8ddaec483dfa0a580a7c8194ee625a6251743859070415aa7f8f384abd6c550"],"id":1 }' localhost:9932
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "node2_grandpa_mnemonics", "0x6725d2323bc3e69d1017a47cefe70a4ee5760ffd4175852370c439132fe06916"],"id":1 }' localhost:9932

# node3
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "node3_babe_mnemonics", "0x6c14813c02fa0b9992560cae02337c748af2e46bb5a1b26b6011bde02f92f356"],"id":1 }' localhost:9934
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "node3_grandpa_mnemonics", "0x06a368a12a24785b2be5f332ae51d947c49d2aac1d8b5804c25a1c47bb838272"],"id":1 }' localhost:9934

# node4
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "node4_babe_mnemonics", "0x966c68c4308b757bef26f21e4951cfd47e6a56ce6c68350dff5d3355bbf27749"],"id":1 }' localhost:9935
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "node4_grandpa_mnemonics", "0x290ed0c0ce03c67d598f31321fe77f79684ffe9cdb5824d02707dc21e1843823"],"id":1 }' localhost:9935

# node5
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "node5_babe_mnemonics", "0x2e6dba967ee6ca20655e92ee82954aed4d88975435a835b97973c270dfa67402"],"id":1 }' localhost:9936
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "node5_grandpa_mnemonics", "0xd76c05af97a59a4a3bb8ccbe5811547e26bc185f3acf7b401ad0e40f17ac880b"],"id":1 }' localhost:9936