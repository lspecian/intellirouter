import { IntelliRouter } from '../src';

// Initialize the client
const client = new IntelliRouter({
    apiKey: process.env.INTELLIROUTER_API_KEY || 'your-api-key',
});

async function main() {
    try {
        console.log('Response:');

        // Create a streaming chat completion
        const stream = await client.chat.createCompletionStream({
            model: 'gpt-3.5-turbo',
            messages: [
                { role: 'system', content: 'You are a helpful assistant.' },
                { role: 'user', content: 'Write a short poem about programming.' },
            ],
        });

        // Process the stream
        for await (const chunk of stream) {
            const content = chunk.choices[0].delta.content;
            if (content) {
                process.stdout.write(content);
            }
        }

        console.log('\n');
    } catch (error) {
        console.error('Error:', error);
    }
}

main();