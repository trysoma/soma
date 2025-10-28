import z from 'zod'
import { createSomaFunction } from '@soma/sdk/bridge';
import type { ProviderController } from '@soma/sdk';

const inputSchema = z.object({
	min: z.number(),
	max: z.number(),
});
const outputSchema = z.object({
	randomNumber: z.number(),
});

export const providerController: ProviderController = {
    typeId: 'random-number',
    name: 'Random Number',
    documentation: 'Generates a random number between two numbers',
    categories: ['random'],
    credentialControllers: [
        {
            type: "Oauth2AuthorizationCodeFlow",
            field0: {
                staticCredentialConfiguration: {
                    authUri: 'https://random-number.com/auth',
                    tokenUri: 'https://random-number.com/token',
                    userinfoUri: 'https://random-number.com/userinfo',
                    jwksUri: 'https://random-number.com/jwks',
                    issuer: 'https://random-number.com',
                    scopes: ['openid', 'profile', 'email'],
                },
            }
        }
    ],
};

export default createSomaFunction({
    inputSchema,
    outputSchema,
    providerController,
    functionController: {
        name: 'random-number',
        description: 'Generates a random number between two numbers',
    },
    handler: async ({min, max})=> {
        const randomNumber = Math.floor(Math.random() * (max - min + 1)) + min;
        return { randomNumber };    
    },
});
