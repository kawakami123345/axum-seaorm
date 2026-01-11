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
            mutator: {
                path: './src/api/custom-fetch.ts',
                name: 'customFetch',
            },
        },
    },
};
