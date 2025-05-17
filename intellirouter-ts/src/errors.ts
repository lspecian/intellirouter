/**
 * Base error class for IntelliRouter errors
 */
export class IntelliRouterError extends Error {
    /**
     * Error code
     */
    public readonly code: string;

    /**
     * HTTP status code (if applicable)
     */
    public readonly status?: number;

    /**
     * Additional error details
     */
    public readonly details?: unknown;

    constructor(message: string, options: { code: string; status?: number; details?: unknown }) {
        super(message);
        this.name = this.constructor.name;
        this.code = options.code;
        this.status = options.status;
        this.details = options.details;

        // Ensure proper prototype chain for instanceof checks
        Object.setPrototypeOf(this, IntelliRouterError.prototype);
    }
}

/**
 * Error thrown when a request to the IntelliRouter API fails
 */
export class ApiError extends IntelliRouterError {
    constructor(message: string, options: { code: string; status: number; details?: unknown }) {
        super(message, options);
        Object.setPrototypeOf(this, ApiError.prototype);
    }
}

/**
 * Error thrown when authentication fails
 */
export class AuthenticationError extends ApiError {
    constructor(message: string, options: { details?: unknown } = {}) {
        super(message, { code: 'authentication_error', status: 401, details: options.details });
        Object.setPrototypeOf(this, AuthenticationError.prototype);
    }
}

/**
 * Error thrown when a request is not authorized
 */
export class AuthorizationError extends ApiError {
    constructor(message: string, options: { details?: unknown } = {}) {
        super(message, { code: 'authorization_error', status: 403, details: options.details });
        Object.setPrototypeOf(this, AuthorizationError.prototype);
    }
}

/**
 * Error thrown when a resource is not found
 */
export class NotFoundError extends ApiError {
    constructor(message: string, options: { details?: unknown } = {}) {
        super(message, { code: 'not_found', status: 404, details: options.details });
        Object.setPrototypeOf(this, NotFoundError.prototype);
    }
}

/**
 * Error thrown when a request times out
 */
export class TimeoutError extends IntelliRouterError {
    constructor(message: string, options: { details?: unknown } = {}) {
        super(message, { code: 'timeout', details: options.details });
        Object.setPrototypeOf(this, TimeoutError.prototype);
    }
}

/**
 * Error thrown when validation fails
 */
export class ValidationError extends IntelliRouterError {
    constructor(message: string, options: { details?: unknown } = {}) {
        super(message, { code: 'validation_error', details: options.details });
        Object.setPrototypeOf(this, ValidationError.prototype);
    }
}

/**
 * Error thrown when the rate limit is exceeded
 */
export class RateLimitError extends ApiError {
    constructor(message: string, options: { details?: unknown } = {}) {
        super(message, { code: 'rate_limit_exceeded', status: 429, details: options.details });
        Object.setPrototypeOf(this, RateLimitError.prototype);
    }
}

/**
 * Error thrown when the server returns an error
 */
export class ServerError extends ApiError {
    constructor(message: string, options: { status: number; details?: unknown }) {
        super(message, { code: 'server_error', status: options.status, details: options.details });
        Object.setPrototypeOf(this, ServerError.prototype);
    }
}