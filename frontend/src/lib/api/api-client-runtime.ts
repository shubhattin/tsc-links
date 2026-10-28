import { api } from '$lib/api/ky';

/**
 * Runtime config for the generated Hey API ky client.
 */
import type { CreateClientConfig } from './generated/client.gen';

export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  ky: api,
});
