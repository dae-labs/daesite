import { box } from "tweetnacl";

/**
 * Generates a NaCl key pair for encryption.
 */
export const generateKeyPair = () => box.keyPair();

/**
 * Converts a Uint8Array to a Base64 string.
 * @param uint8Array The array to convert.
 */
export const toBase64 = (uint8Array: Uint8Array): string => {
  return btoa(String.fromCharCode(...uint8Array));
};
