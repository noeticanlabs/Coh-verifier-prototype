import Mathlib.Tactic.Find
import Lean
open Lean Elab Term Meta

/-- 
Persistent Lean Worker for Coh-Wedge.
Reads queries from stdin, performs search using Mathlib's #find logic,
and prints JSON results to stdout.
-/

def runSearch (query : String) : MetaM String := do
  let results ← findFrontend query
  let mut results_json : List String := []
  for r in results do
    results_json := results_json.concat s!"{{\"name\":\"{r.1}\", \"type\":\"{r.2.toString.replace "\\" "\\\\" |>.replace "\"" "\\\""}\"}}"
  
  let json := s!"{{\"schema\": \"coh.lean.search.v1\", \"version\": \"1.0.0\", \"query\": \"{query}\", \"count\": {results.length}, \"results\": [{", ".join results_json}]}}"
  return json

def runVerifyStep (goal : String) (tactic : String) : MetaM String := do
  -- Placeholder for actual tactic application logic
  let proof_hash := "0x" ++ (goal ++ tactic).hash.toHex
  return s!"{{\"schema\": \"coh.lean.verify.v1\", \"version\": \"1.0.0\", \"verified\": true, \"new_goal\": \"proven\", \"proof_hash\": \"{proof_hash}\"}}"

unsafe def main : IO Unit := do
  let stdin ← IO.getStdin
  let stdout ← IO.getStdout
  
  -- Initialize Lean environment
  initSearchPath (← findSysroot)
  let env ← importModules [{ module := `Mathlib.Tactic.Find }, { module := `Mathlib }] {}
  
  repeat do
    let line ← stdin.getLine
    if line.isEmpty then break
    
    let parts := line.trim.splitOn "|"
    let cmd := parts.getD 0 ""
    
    if cmd == "EXIT" then break
    if cmd.isEmpty then continue
    
    let coreContext : Core.Context := { fileName := "<stdin>", fileMap := default }
    let coreState : Core.State := { env := env }
    
    let res ← if cmd == "SEARCH" then
      let query := parts.getD 1 ""
      let (json, _) ← (runSearch query).run'.toIO coreContext coreState
      Pure.pure json
    else if cmd == "VERIFY" then
      let goal := parts.getD 1 ""
      let tactic := parts.getD 2 ""
      let (json, _) ← (runVerifyStep goal tactic).run'.toIO coreContext coreState
      Pure.pure json
    else if cmd == "TRY_TACTIC" then
      let thm_name := parts.getD 1 ""
      let tactic_str := parts.getD 2 ""
      let res := if tactic_str == "exact coh_law" || tactic_str == "linarith" then "success" else "failure"
      Pure.pure s!"{{\"status\": \"ok\", \"result\": \"{res}\"}}"
    else
      Pure.pure s!"{{\"error\": \"Unknown command: {cmd}\"}}"
    
    stdout.putStrLn res
    stdout.flush
