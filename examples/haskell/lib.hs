module Lib where

foreign export ccall on_request :: Int -> IO ()
on_request ptr = do
  print ptr
  putStrLn "--------"

main = do
  putStrLn "HELLO WORLLD"

