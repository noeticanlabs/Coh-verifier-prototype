import Mathlib.Order.Basic
import Coh.Boundary.RationalInf
open Coh.Boundary
theorem repair_test (a b c d : NNRat) : a + b ≤ c + d := by apply add_le_add_typo
