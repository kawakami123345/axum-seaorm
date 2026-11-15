module.exports = {
    bookkeeper: {
        input: {
            target: '../openapi.json',
            validation: false,
        },
        output: {
            mode: 'tags-split',
            target: 'src/api/endpoints',
            schemas: 'src/api/model',
            client: 'react-query',
            httpClient: 'fetch',
            clean: true,
            override: {
                mutator: {
                    path: './src/lib/custom-fetch.ts',
                    name: 'customFetch',
                },
            },
        },
    },
};
