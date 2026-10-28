/**
 * App-facing API layer: nested SDK on a single ky client.
 */
import { client as httpClient } from './generated/client.gen';
import { Api } from './generated/sdk.gen';

export const client = new Api({ client: httpClient });

export type * from './generated/types.gen';
export * from './generated/zod.gen';
