/* tslint:disable */
/* eslint-disable */
export * from './runtime';
export * from './models/index';

import { Configuration } from '@soma/sdk/bridge';
import { DefaultApi } from './apis/DefaultApi';
export const bridge = ()=> new DefaultApi(new Configuration({
    basePath: process.env.SOMA_SERVER_BASE_URL || 'http://localhost:3000',
}));