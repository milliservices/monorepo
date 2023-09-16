module Lib where

import Control.Applicative ((<$>), (<*>))
import Data.ByteString qualified as ByteString
import Data.ByteString.Char8 (ByteString, pack, unpack)
import Data.Int (Int32)
import Data.Word (Word8)
import Foreign.C.Types (CInt)
import Foreign.Marshal.Alloc (allocaBytes)
import Foreign.Marshal.Array (allocaArray, peekArray, pokeArray)
import Foreign.Ptr (IntPtr, Ptr, intPtrToPtr, ptrToIntPtr)
import Foreign.Storable (Storable, peek, peekByteOff, poke, pokeByteOff, sizeOf)

foreign import ccall sendResponse :: Int32 -> IO ()

foreign import ccall setResponseMetadata :: Int32 -> Int32 -> IO ()

data SizedPtrData = SizedPtrData Int32 Int32
  deriving (Show)

dataPtrSize = sizeOf (undefined :: Int32)

dataLenSize = sizeOf (undefined :: Int32)

instance Storable SizedPtrData where
  sizeOf _ = dataPtrSize + dataLenSize

  peek ptr =
    SizedPtrData
      <$> peekByteOff ptr 0
      <*> peekByteOff ptr dataPtrSize

  poke ptr (SizedPtrData dataPtr dataSize) = do
    pokeByteOff ptr 0 dataPtr
    pokeByteOff ptr dataPtrSize dataSize

readFromMemory :: Ptr SizedPtrData -> IO [Word8]
readFromMemory ptr = do
  sizedPtr <- peek ptr
  let SizedPtrData dataPtrInt dataLen = sizedPtr
  let dataPtr = intPtrToPtr $ fromIntegral dataPtrInt
  peekArray (fromIntegral dataLen) dataPtr

writeToMemory :: [Word8] -> IO (Ptr SizedPtrData)
writeToMemory buffer = do
  let dataLen = length buffer
  dataPtr <- allocaArray dataLen (\ptr -> ptr <$ pokeArray ptr buffer)
  allocaBytes
    (sizeOf (undefined :: SizedPtrData))
    ( \ptr -> do
        poke ptr $ SizedPtrData (fromIntegral $ ptrToIntPtr dataPtr) (fromIntegral dataLen)
        pure ptr
    )

ptrToInt :: Ptr a -> Int32
ptrToInt = fromIntegral . ptrToIntPtr

foreign export ccall onRequest :: Ptr SizedPtrData -> IO ()

onRequest ptr = do
  bytes <- readFromMemory ptr
  let str = unpack $ ByteString.pack bytes
  putStrLn $ ":: [HASKELL] " ++ str

  responsePtr <- ptrToInt <$> writeToMemory (ByteString.unpack $ pack "Hello world")
  sendResponse responsePtr

  keyPtr <- ptrToInt <$> writeToMemory (ByteString.unpack $ pack "Server")
  valuePtr <- ptrToInt <$> writeToMemory (ByteString.unpack $ pack "foobar")
  setResponseMetadata keyPtr valuePtr
