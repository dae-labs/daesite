import { encode, decode } from "msgpack-lite";
import { unzlibSync, zlibSync } from "fflate";

/**
 * Compresses data by encoding it to MsgPack and then compressing it with zlib.
 * @param data The data to compress.
 */
const compressData = (data: any): Uint8Array => {
  const encodedData = encode(data);
  return zlibSync(encodedData);
};

/**
 * Decompresses zlib-compressed data and decodes it from MsgPack format.
 * @param data The data to decompress.
 */
const decompressData = (data: Uint8Array): any => {
  try {
    const decompressedData = unzlibSync(data);
    return decode(decompressedData);
  } catch (error) {
    console.error("Decompression or decoding failed:", error);
    throw error;
  }
};

export {compressData, decompressData};
