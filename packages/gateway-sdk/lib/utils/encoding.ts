/**
 * Converts a Uint8Array to a Base64 string.
 * @param uint8Array The array to convert.
 */
const uint8ArrayToBase64 = (uint8Array: Uint8Array): string => {
  return btoa(String.fromCharCode(...uint8Array));
};

export {uint8ArrayToBase64};
