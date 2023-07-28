
export const USDT_LOCATION = {
    V1: {
        parents: 1,
        interior: {
            X3: [
                {
                    Parachain: 1000
                },
                {
                    PalletInstance: 50
                },
                {
                    GeneralIndex: 1984
                }
            ]
        }
    }
};

export const USDT_METADATA = {
    metadata: {
        name: "Tether USD",
        symbol: "USDT",
        decimals: 6,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};

export const USDC_LOCATION = {
    V1: {
        parents: 1,
        interior: {
            X3: [
                {
                    Parachain: 1000
                },
                {
                    PalletInstance: 50
                },
                {
                    GeneralIndex: 1985
                }
            ]
        }
    }
};

export const USDC_METADATA = {
    metadata: {
        name: "USDC",
        symbol: "USDC",
        decimals: 10,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};

export const MANDEX_METADATA = {
    metadata: {
        name: "MANDEX",
        symbol: "MANDEX",
        decimals: 18,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};

export const LP_USDT_USDC_METADATA = {
    metadata: {
        name: "LP-USDC-USDT",
        symbol: "LP",
        decimals: 12,
        isFrozen: false
    },
    minBalance: 1,
    isSufficient: true
};