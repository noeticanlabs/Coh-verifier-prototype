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

unsafe def main : IO Unit := do
  let stdin ← IO.getStdin
  let stdout ← IO.getStdout
  
  -- Initialize Lean environment (simplified for persistent worker)
  initSearchPath (← findSysroot)
  let env ← importModules [{ module := `Mathlib.Tactic.Find }] {}
  
  repeat do
    let line ← stdin.getLine
    if line.isEmpty then break
    
    let query := line.trim
    if query == "EXIT" then break
    if query.isEmpty then continue
    
    -- Execute search in MetaM context
    let coreContext : Core.Context := { fileName := "<stdin>", fileMap := default }
    let coreState : Core.State := { env := env }
    
    let (res, _) ← (runSearch query).run'.toIO coreContext coreState
    
    stdout.putStrLn res
    stdout.flush
