import { IntelliRouter } from '../src';

// Initialize the client
const client = new IntelliRouter({
    apiKey: process.env.INTELLIROUTER_API_KEY || 'your-api-key',
});

async function main() {
    try {
        // Create a chat completion
        const completion = await client.chat.createCompletion({
            model: 'gpt-3.5-turbo',
            messages: [
                { role: 'system', content: 'You are a helpful assistant.' },
                { role: 'user', content: 'Hello, how are you?' },
            ],
        });

        console.log('Response:');
        console.log(completion.choices[0].message.content);
        console.log('\nUsage:');
        console.log(`Prompt tokens: ${completion.usage.prompt_tokens}`);
        console.log(`Completion tokens: ${completion.usage.completion_tokens}`);
        console.log(`Total tokens: ${completion.usage.total_tokens}`);
    } catch (error) {
        console.error('Error:', error);
    }
}

main();