import Coh.Prelude

namespace Coh.Kernel

universe u

/-- Core Receipt structure representing a discrete state transition. -/
structure Receipt where
  pre : R
  post : R
  spend : R
  defect : R
  authority : R := 0

/-- Morphism mapping between state spaces. -/
def map (f : R ? R) (r : Receipt) : Receipt where
  pre := f r.pre
  post := f r.post
  spend := r.spend
  defect := r.defect
  authority := r.authority

/-- Lemma A.1.1: map_pre -/
lemma map_pre (f : R ? R) (r : Receipt) : (map f r).pre = f r.pre := rfl

/-- Lemma A.1.2: map_post -/
lemma map_post (f : R ? R) (r : Receipt) : (map f r).post = f r.post := rfl

/-- Lemma A.1.3: map_spend -/
lemma map_spend (f : R ? R) (r : Receipt) : (map f r).spend = r.spend := rfl

/-- Lemma A.1.4: map_defect -/
lemma map_defect (f : R ? R) (r : Receipt) : (map f r).defect = r.defect := rfl

end Coh.Kernel
