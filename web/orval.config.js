export default {
    'bookkeeper-api': {
        input: {
            target: '../openapi.json',
            validation: false
        },
        output: {
            mode: 'tags-split',
            target: 'src/api/endpoints',
            schemas: 'src/api/model',
            client: 'react-query',
            httpClient: 'fetch',
        },
    },
};

