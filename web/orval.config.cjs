// AJV 8 is a direct dev dependency so Orval's draft-04 parser can resolve its peer.
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
            formatter: 'prettier',
            override: {
                mutator: {
                    path: './src/lib/custom-fetch.ts',
                    name: 'customFetch',
                },
            },
        },
    },
};
