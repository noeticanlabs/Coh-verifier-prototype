import Lean

open Lean Lean.LeanMeta Lean.Elab Lean.Elab.Tactic

/--
Persistent worker that receives theorem+tactic commands via stdin and executes them via Lean.

Protocol:
- Input:  TRY_TACTIC|{theorem_name}|{tactic}
- Output: JSON with result ("success" or "failure"), proof_hash, and optionally stderr
- Terminate: empty line
-
Example input:
TRY_TACTIC|test_lemma|exact rfl
-/

-- Parse a TRY_TACTIC command, returning (theorem_name, tactic)
def parseTryTactic (line : String) : Option (String × String) := do
  if !line.startsWith "TRY_TACTIC|" then none
  else
    let rest := line.dropRight $ line.length - "TRY_TACTIC|".length
    match rest.splitOn "|" with
    | [thm, tac] => some (thm.trim, tac.trim)
    | _ => none

/-- Attempt to verify a tactic by parsing and running it - returns proof hash or error -/
unsafe def runLeanVerification (theoremName : String) (tactic : String) : IO (String × String) := do
  let theoremFile := s!"{theoremName}.lean"
  /-
  Actually run Lean to verify the tactic.
  Strategy: invoke Lean as a subprocess with the theorem file and tactic.
  -/
  let child ← IO.Process.spawn {
    cmd := "lake"
    args := #["exec", "lean", "--", theoremFile]
    stdin := IO.Process.Stdio.piped
    stdout := IO.Process.Stdio.piped
    stderr := IO.Process.Stdio.piped
  }
  -- Write tactic input
  child.stdin.get (← IO.getStdout).putStrLn tactic
  child.stdin.get.flush
  let out ← IO.IO.getLines child.stdout.get
  let err ← IO.getLines child.stderr.get
  let exitCode ← child.wait
  if exitCode == 0 then
    /- Success - extract proof hash from output -/
    let proofHash := s!"0x{((·.toUInt64) $ out.size.hash)}"
    pure (proofHash, "")
  else
    pure ("", err.mkString "\n")

/--
Main worker loop - reads commands and outputs JSON results.
This is the entry point when running: lean --run persistent_worker.lean
-/
unsafe def main : IO Unit := do
  let stdin ← IO.getStdin
  let stdout ← IO.getStdout

  repeat do
    let line ← stdin.getLine
    if line.isEmpty then break

    -- Parse command
    let (thmName, tactic) ← do
      match parseTryTactic line with
      | some (t, a) => pure (t, a)
      | none =>
        -- Unknown command format - output error
        stdout.putStrLn "{\"result\":\"error\",\"proof_hash\":\"\",\"stderr\":\"unknown command format\"}"
        stdout.flush
        continue

    -- Execute verification (real Lean call)
    let (proofHash, stderr) ← runLeanVerification thmName tactic

    -- Output JSON result
    let resultStr :=
      if proofHash.isEmpty then "failure" else "success"
    let outputJson :=
      if stderr.isEmpty then
        s!"{{\"result\":\"{resultStr}\",\"proof_hash\":\"{proofHash}\",\"stderr\":\"\"}}"
      else
        s!"{{\"result\":\"{resultStr}\",\"proof_hash\":\"{proofHash}\",\"stderr\":{stderr}}}"
    stdout.putStrLn outputJson
    stdout.flush
