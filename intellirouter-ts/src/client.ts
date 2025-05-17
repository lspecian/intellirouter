import { IntelliRouterConfig } from './types';
import { HttpTransport } from './transport';
import { ConfigManager } from './config';
import { ChatClient } from './chat';
import { ChainClient } from './chains';

/**
 * Main client for interacting with IntelliRouter
 */
export class IntelliRouter {
    private readonly transport: HttpTransport;
    private readonly _config: ConfigManager;
    private readonly _chat: ChatClient;
    private readonly _chains: ChainClient;

    /**
     * Create a new IntelliRouter client
     * @param config Client configuration
     */
    constructor(config: IntelliRouterConfig = {}) {
        // Initialize transport
        this.transport = new HttpTransport(config);

        // Initialize config manager
        this._config = new ConfigManager(this.transport, config);

        // Load configuration from environment variables
        this._config.loadFromEnvironment();

        // Initialize clients
        this._chat = new ChatClient(this.transport);
        this._chains = new ChainClient(this.transport);
    }

    /**
     * Chat completions API
     */
    public get chat(): ChatClient {
        return this._chat;
    }

    /**
     * Chain execution API
     */
    public get chains(): ChainClient {
        return this._chains;
    }

    /**
     * Configuration management
     */
    public get config(): ConfigManager {
        return this._config;
    }

    /**
     * Get the current configuration
     * @returns Current configuration
     */
    public getConfig(): IntelliRouterConfig {
        return this._config.getConfig();
    }

    /**
     * Update the configuration
     * @param config New configuration
     * @returns Updated configuration
     */
    public updateConfig(config: Partial<IntelliRouterConfig>): IntelliRouterConfig {
        return this._config.updateConfig(config);
    }
}