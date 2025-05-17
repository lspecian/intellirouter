import { IntelliRouter } from '../client';
import { HttpTransport } from '../transport';
import { ChatClient } from '../chat';
import { ChainClient } from '../chains';
import { ConfigManager } from '../config';

// Mock dependencies
jest.mock('../transport/http');
jest.mock('../chat/client');
jest.mock('../chains/client');
jest.mock('../config');

describe('IntelliRouter', () => {
    beforeEach(() => {
        jest.clearAllMocks();
    });

    it('should initialize with default config', () => {
        const client = new IntelliRouter();

        expect(HttpTransport).toHaveBeenCalledWith({});
        expect(ConfigManager).toHaveBeenCalled();
        expect(ChatClient).toHaveBeenCalled();
        expect(ChainClient).toHaveBeenCalled();

        expect(client).toBeDefined();
        expect(client.chat).toBeDefined();
        expect(client.chains).toBeDefined();
        expect(client.config).toBeDefined();
    });

    it('should initialize with custom config', () => {
        const config = {
            apiKey: 'test-key',
            baseUrl: 'https://example.com',
            timeout: 5000,
            maxRetries: 2,
        };

        const client = new IntelliRouter(config);

        expect(HttpTransport).toHaveBeenCalledWith(config);
        expect(ConfigManager).toHaveBeenCalledWith(expect.anything(), config);
        expect(ChatClient).toHaveBeenCalled();
        expect(ChainClient).toHaveBeenCalled();

        expect(client).toBeDefined();
        expect(client.chat).toBeDefined();
        expect(client.chains).toBeDefined();
        expect(client.config).toBeDefined();
    });

    it('should expose chat API', () => {
        const client = new IntelliRouter();
        const mockChatClient = (ChatClient as jest.Mock).mock.instances[0];

        expect(client.chat).toBe(mockChatClient);
    });

    it('should expose chains API', () => {
        const client = new IntelliRouter();
        const mockChainClient = (ChainClient as jest.Mock).mock.instances[0];

        expect(client.chains).toBe(mockChainClient);
    });

    it('should expose config API', () => {
        const client = new IntelliRouter();
        const mockConfigManager = (ConfigManager as jest.Mock).mock.instances[0];

        expect(client.config).toBe(mockConfigManager);
    });

    it('should delegate getConfig to config manager', () => {
        const client = new IntelliRouter();
        const mockConfigManager = (ConfigManager as jest.Mock).mock.instances[0];
        mockConfigManager.getConfig = jest.fn().mockReturnValue({ apiKey: 'test-key' });

        const config = client.getConfig();

        expect(mockConfigManager.getConfig).toHaveBeenCalled();
        expect(config).toEqual({ apiKey: 'test-key' });
    });

    it('should delegate updateConfig to config manager', () => {
        const client = new IntelliRouter();
        const mockConfigManager = (ConfigManager as jest.Mock).mock.instances[0];
        mockConfigManager.updateConfig = jest.fn().mockReturnValue({ apiKey: 'new-key' });

        const config = client.updateConfig({ apiKey: 'new-key' });

        expect(mockConfigManager.updateConfig).toHaveBeenCalledWith({ apiKey: 'new-key' });
        expect(config).toEqual({ apiKey: 'new-key' });
    });
});