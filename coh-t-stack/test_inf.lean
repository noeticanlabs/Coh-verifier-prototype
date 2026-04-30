import Mathlib

abbrev ENNRat := WithTop NNRat

structure IsRationalInf (s : Set ENNRat) (i : ENNRat) : Prop where
  left : ∀ x ∈ s, i ≤ x
  greatest : ∀ k, (∀ x ∈ s, k ≤ x) → k ≤ i

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
