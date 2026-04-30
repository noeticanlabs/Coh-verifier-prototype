import Mathlib

namespace Coh.Boundary

abbrev ENNRat := WithTop NNRat

structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  left : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

/--
## Lemma: Pairwise Add Infimum (Lower Bound)
For any z in (s1 + s2), we have i1 + i2 ≤ z.
This is the easy half: element-wise bound from each infimum.
-/
theorem isRationalInf_add_lower (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2)
  (z : ENNRat) (hz : z ∈ { z | ∃ x ∈ s1, ∃ y ∈ s2, z = x + y }) :
  i1 + i2 ≤ z := by
  obtain ⟨x, hx, y, hy, rfl⟩ := hz
  exact add_le_add (h1.left x hx) (h2.left y hy)

/--
## Lemma: Pairwise Add Infimum (Greatest Lower Bound)
If k is a lower bound for (s1 + s2), then k ≤ i1 + i2.
This uses the universal lower bound property of each infimum.
-/
theorem isRationalInf_add_greatest (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2)
  (k : ENNRat) (hk : ∀ z, (∃ x ∈ s1, ∃ y ∈ s2, z = x + y) → k ≤ z) :
  k ≤ i1 + i2 := by
  have h_k_le : ∀ x ∈ s1, ∀ y ∈ s2, k ≤ x + y := by
    intro x hx y hy
    exact hk (x + y) ⟨x, hx, y, hy, rfl⟩
  have h2_ineq : ∀ x ∈ s1, k - x ≤ i2 := by
    intro x hx
    apply h2.greatest
    intro y hy
    have h_add := h_k_le x hx y hy
    rw [add_comm] at h_add
    exact tsub_le_iff_right.mpr h_add
  have h1_ineq : k - i2 ≤ i1 := by
    apply h1.greatest
    intro x hx
    have h_kx := h2_ineq x hx
    have h_k_le_x_i2 : k ≤ i2 + x := tsub_le_iff_right.mp h_kx
    rw [add_comm] at h_k_le_x_i2
    exact tsub_le_iff_right.mpr h_k_le_x_i2
  have h_final : k ≤ i1 + i2 := by
    have h_k_le_i1_i2 : k ≤ i1 + i2 := tsub_le_iff_right.mp h1_ineq
    exact h_k_le_i1_i2
  exact h_final

/--
## Theorem: Pairwise Add Infimum
The infimum of (s1 + s2) is i1 + i2.
[PROVED] — both the lower bound and the greatest lower bound.
-/
theorem isRationalInf_add_inf_le (s1 s2 : Set ENNRat) (i1 i2 : ENNRat)
  (h1 : IsRationalInf s1 i1) (h2 : IsRationalInf s2 i2) :
  IsRationalInf (fun z => ∃ x ∈ s1, ∃ y ∈ s2, z = x + y) (i1 + i2) := by
  constructor
  · intro z hz
    exact isRationalInf_add_lower s1 s2 i1 i2 h1 h2 z hz
  · intro k hk
    exact isRationalInf_add_greatest s1 s2 i1 i2 h1 h2 k hk

end Coh.Boundary
