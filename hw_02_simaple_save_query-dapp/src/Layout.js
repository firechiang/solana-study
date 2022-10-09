// @flow

import * as BufferLayout from '@solana/buffer-layout';

/**
 * Layout for a public key
 */
export function publicKey(property) {
    return BufferLayout.blob(32, property);
}

export const messagSpace = 32+1+256;
