import Mathlib

theorem gradient_descent_terminates
  (M : ℕ → ℕ)
  (h_strict_decrease : ∀ n, M (n + 1) < M n ∨ M n = 0)
  (g₀ : ℕ) :
  ∃ n, M (g₀ + n) = 0 :=
by
  generalize hm : M g₀ = m
  induction m using Nat.strong_induction_on generalizing g₀
  case h m ih =>
    by_cases h0 : M g₀ = 0
    · use 0; exact h0
    · have h_next := h_strict_decrease g₀
      rcases h_next with h_dec | h_zero
      · have h_lt : M (g₀ + 1) < m := by rw [← hm]; exact h_dec
        rcases ih (M (g₀ + 1)) h_lt (g₀ + 1) rfl with ⟨n, hn⟩
        use n + 1
        have h_eq : g₀ + (n + 1) = g₀ + 1 + n := by omega
        rw [h_eq]
        exact hn
      · contradiction
