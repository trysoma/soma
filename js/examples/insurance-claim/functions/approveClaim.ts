import { createSomaFunction } from '@soma/sdk/bridge';
import type { ProviderController } from '@soma/sdk';
import { assessmentSchema } from '../agents';
import z from 'zod';
export const providerController: ProviderController = {
    typeId: 'approve-claim',
    name: 'Approve Claim',
    documentation: 'Approve a claim',
    categories: [],
    credentialControllers: [
        // {
        //     type: "Oauth2AuthorizationCodeFlow",
        //     field0: {
        //         staticCredentialConfiguration: {
        //             authUri: 'https://random-number.com/auth',
        //             tokenUri: 'https://random-number.com/token',
        //             userinfoUri: 'https://random-number.com/userinfo',
        //             jwksUri: 'https://random-number.com/jwks',
        //             issuer: 'https://random-number.com',
        //             scopes: ['openid', 'profile', 'email'],
        //         },
        //     }
        // }
        {
            type: "NoAuth",
        }
    ],
};

export default createSomaFunction({
    inputSchema: assessmentSchema,
    outputSchema: z.object({
        approved: z.boolean(),
    }),
    providerController,
    functionController: {
        name: 'approve-claim',
        description: 'Approve a claim',
    },
    handler: async ({claim})=> {
        console.log('Claim', claim);
        // perform an async action here to approve the claim
        return { approved: true };    
    },
});
