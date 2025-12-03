import 'react-native-gesture-handler/jestSetup';

// Mock react-native-keychain
jest.mock('react-native-keychain', () => ({
  setGenericPassword: jest.fn().mockResolvedValue(true),
  getGenericPassword: jest.fn().mockResolvedValue({
    username: 'test',
    password: 'test',
  }),
  resetGenericPassword: jest.fn().mockResolvedValue(true),
  getSupportedBiometryType: jest.fn().mockResolvedValue('FaceID'),
  ACCESSIBLE: {
    WHEN_UNLOCKED_THIS_DEVICE_ONLY: 'WhenUnlockedThisDeviceOnly',
  },
  AUTHENTICATION_TYPE: {
    BIOMETRICS: 'Biometrics',
  },
}));

// Mock react-native-encrypted-storage
jest.mock('react-native-encrypted-storage', () => ({
  setItem: jest.fn().mockResolvedValue(true),
  getItem: jest.fn().mockResolvedValue('{}'),
  removeItem: jest.fn().mockResolvedValue(true),
}));

// Mock @polkadot/api
jest.mock('@polkadot/api', () => ({
  ApiPromise: {
    create: jest.fn().mockResolvedValue({
      isReady: Promise.resolve(),
      query: {
        system: {
          account: jest.fn().mockResolvedValue({
            data: {
              free: { toString: () => '1000000000000000000' },
            },
          }),
        },
      },
    }),
  },
  WsProvider: jest.fn(),
}));

// Mock react-native-reanimated
jest.mock('react-native-reanimated', () => {
  const Reanimated = require('react-native-reanimated/mock');
  Reanimated.default.call = () => {};
  return Reanimated;
});

// Silence the warning: Animated: `useNativeDriver` is not supported
jest.mock('react-native/Libraries/Animated/NativeAnimatedHelper');
