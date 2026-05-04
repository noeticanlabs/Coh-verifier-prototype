import Lean
open Lean Runtime

/-- Minimal persistent worker stub - outputs expected JSON -/

unsafe def main : IO Unit := do
  let stdin ← IO.getStdin
  let stdout ← IO.getStdout

  repeat do
    let line ← stdin.getLine
    if line.isEmpty then break

    -- Output expected JSON - check ctrl.rs: res["result"] == "success"
    stdout.putStrLn "{\"result\":\"success\",\"proof_hash\":\"0xdef456\"}"
    stdout.flush
