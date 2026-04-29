import Mathlib

namespace Coh.Boundary

/-- Extended Non-Negative Rationals: NNRat with an infinity element ⊤ -/
abbrev ENNRat := WithTop NNRat

/--
A set s has rational infimum i if:
1. i is a lower bound of s
2. Any lower bound k satisfies k ≤ i
-/
structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  left : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

/-- 
Theorem: The infimum of the pairwise sum of two sets is the sum of their infima.
`inf (s1 + s2) = inf s1 + inf s2`
-/
theorem isRationalInf_add_inf_le (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  sorry

end Coh.Boundary
