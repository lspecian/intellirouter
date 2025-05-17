import { IntelliRouterConfig } from '../types';
import { Transport } from '../transport';

/**
 * Configuration manager for IntelliRouter
 */
export class ConfigManager {
    private readonly transport: Transport;
    private config: IntelliRouterConfig;

    /**
     * Create a new configuration manager
     * @param transport Transport layer
     * @param config Initial configuration
     */
    constructor(transport: Transport, config: IntelliRouterConfig = {}) {
        this.transport = transport;
        this.config = config;
    }

    /**
     * Get the current configuration
     * @returns Current configuration
     */
    public getConfig(): IntelliRouterConfig {
        return { ...this.config };
    }

    /**
     * Update the configuration
     * @param config New configuration
     * @returns Updated configuration
     */
    public updateConfig(config: Partial<IntelliRouterConfig>): IntelliRouterConfig {
        this.config = {
            ...this.config,
            ...config,
        };

        return this.getConfig();
    }

    /**
     * Get a configuration value
     * @param key Configuration key
     * @param defaultValue Default value if the key is not found
     * @returns Configuration value
     */
    public get<K extends keyof IntelliRouterConfig>(key: K, defaultValue?: IntelliRouterConfig[K]): IntelliRouterConfig[K] {
        return this.config[key] !== undefined ? this.config[key] : defaultValue;
    }

    /**
     * Set a configuration value
     * @param key Configuration key
     * @param value Configuration value
     * @returns Updated configuration
     */
    public set<K extends keyof IntelliRouterConfig>(key: K, value: IntelliRouterConfig[K]): IntelliRouterConfig {
        this.config[key] = value;
        return this.getConfig();
    }

    /**
     * Load configuration from environment variables
     * @returns Updated configuration
     */
    public loadFromEnvironment(): IntelliRouterConfig {
        const config: Partial<IntelliRouterConfig> = {};

        if (typeof process !== 'undefined' && process.env) {
            if (process.env.INTELLIROUTER_API_KEY) {
                config.apiKey = process.env.INTELLIROUTER_API_KEY;
            }

            if (process.env.INTELLIROUTER_BASE_URL) {
                config.baseUrl = process.env.INTELLIROUTER_BASE_URL;
            }

            if (process.env.INTELLIROUTER_TIMEOUT) {
                const timeout = parseInt(process.env.INTELLIROUTER_TIMEOUT, 10);
                if (!isNaN(timeout)) {
                    config.timeout = timeout;
                }
            }

            if (process.env.INTELLIROUTER_MAX_RETRIES) {
                const maxRetries = parseInt(process.env.INTELLIROUTER_MAX_RETRIES, 10);
                if (!isNaN(maxRetries)) {
                    config.maxRetries = maxRetries;
                }
            }
        }

        return this.updateConfig(config);
    }

    /**
     * Reset configuration to defaults
     * @returns Default configuration
     */
    public resetToDefaults(): IntelliRouterConfig {
        this.config = {
            baseUrl: 'http://localhost:8000',
            timeout: 30000,
            maxRetries: 3,
        };

        return this.getConfig();
    }
}