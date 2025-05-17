import { IntelliRouter } from '../src';
import {
    ApiError,
    AuthenticationError,
    ValidationError,
    RateLimitError,
    ServerError,
} from '../src/errors';

// Initialize the client with an invalid API key
const client = new IntelliRouter({
    apiKey: 'invalid-api-key',
});

async function main() {
    try {
        // This will fail due to the invalid API key
        const completion = await client.chat.createCompletion({
            model: 'gpt-3.5-turbo',
            messages: [
                { role: 'user', content: 'Hello' },
            ],
        });

        console.log(completion);
    } catch (error) {
        console.error('An error occurred:');

        if (error instanceof ValidationError) {
            console.error('Validation error:', error.message);
        } else if (error instanceof AuthenticationError) {
            console.error('Authentication error:', error.message);
            console.error('Please check your API key.');
        } else if (error instanceof RateLimitError) {
            console.error('Rate limit exceeded:', error.message);
            console.error('Please try again later.');
        } else if (error instanceof ServerError) {
            console.error('Server error:', error.message);
            console.error('Status:', error.status);
        } else if (error instanceof ApiError) {
            console.error('API error:', error.message);
            console.error('Status:', error.status);
            console.error('Details:', error.details);
        } else {
            console.error('Unknown error:', error);
        }
    }
}

main();