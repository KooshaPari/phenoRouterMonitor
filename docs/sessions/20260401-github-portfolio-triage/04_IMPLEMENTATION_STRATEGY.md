# Implementation Strategy

1. classify the visible repo inventory into maintenance cohorts
2. execute only low-regret archive mutations first
3. normalize active-repo rulesets where the current remote state is clearly broken or duplicated
4. extend the normalized baseline to sampled active repos that still have obvious governance gaps
5. roll out only high-confidence reusable repo-local security/governance workflows
6. record every remote mutation and checked-in baseline adjustment in shelf-owned artifacts before
   widening the scope
