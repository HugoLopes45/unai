# Text patterns

This file covers prose text patterns that signal LLM-generated writing. They are not always wrong individually, but they cluster. Real writing has friction, specificity, and voice. These patterns are smooth, abstract, and interchangeable.

How to use: scan text for each pattern below. Fix the ones you find. You do not need to fix all of them — one or two per paragraph is enough to break the LLM signature.

---

## 1. Significance inflation

LLMs signal importance with inflated phrasing because training data rewards gravitas.

Before: "This marks a pivotal moment and stands as a testament to the team's dedication."
After: "The team shipped it on time."

Rule: Replace "pivotal", "testament to", "marks a shift", and "landmark" with a concrete fact about what actually changed.

---

## 2. Notability name-dropping

Listing prestigious outlets or names without quoting or linking to them — presence implies credibility that isn't there.

Before: "Covered by the New York Times, BBC, and Wired, the project attracted widespread attention."
After: "Wired ran a 2,000-word profile in March. The others haven't covered it."

Rule: Name only sources you can cite; remove the rest or replace with the actual claim they would support.

---

## 3. Superficial -ing analyses

Stacking participial phrases ("symbolizing... reflecting... showcasing...") that interpret rather than describe.

Before: "The design uses blue throughout, symbolizing trust and reflecting the brand's commitment to transparency, showcasing its maturity."
After: "The design uses blue throughout. The brand guidelines call it 'trust blue'."

Rule: Cut every -ing clause that explains what something means; leave only what it is.

---

## 4. Promotional language

Travel-brochure adjectives applied to anything the LLM is asked to describe positively.

Before: "Nestled within the breathtaking landscape of the Pacific Northwest, the campus offers a vibrant and inspiring environment."
After: "The campus is in Bellingham, Washington, near Mount Baker."

Rule: Remove all adjectives that belong in a tourism ad; if you need to praise something, name the specific thing worth praising.

---

## 5. Vague attributions

Phantom experts invoked to add authority without naming a single one.

Before: "Experts believe this approach could reshape the industry. Many argue the change is overdue."
After: "Jensen Huang said in the Q3 call that this would replace their existing pipeline by 2026."

Rule: Name the person, the publication, and the date, or cut the attribution entirely.

---

## 6. Formulaic challenges

A boilerplate adversity clause followed by boilerplate resilience, adding no information.

Before: "Despite facing significant challenges, the organization continues to thrive and push forward."
After: "They lost two lead engineers in January and missed the Q1 deadline. They shipped in April."

Rule: Name the challenge and the outcome; delete "despite challenges" and "continues to thrive" as a unit.

---

## 7. AI vocabulary

A lexicon so overused in LLM output that its presence alone flags the source.

Words to remove: tapestry, testament, delve, underscore, pivotal, comprehensive, multifaceted, evolving landscape, vibrant, crucial, moreover, furthermore, in conclusion, ingrained, indelible.

Before: "Furthermore, this multifaceted approach underscores the team's comprehensive commitment to the evolving landscape of data privacy."
After: "The team updated their data retention policy and hired a privacy counsel."

Rule: Search the text for every word in this list and delete or replace each one.

---

## 8. Copula avoidance

Using "serves as", "represents", "embodies", or "boasts" instead of "is" — a habit from writing that tries to avoid repetition of the verb "to be".

Before: "The library serves as a comprehensive resource and represents a significant achievement in open-source collaboration."
After: "The library is a well-maintained open-source package with 4,000 contributors."

Rule: Rewrite "serves as X" and "represents X" as "is X"; use "is" directly.

---

## 9. Negative parallelisms

A two-part rhetorical structure that sounds emphatic but says the same thing twice.

Before: "It's not just a tool for developers — it's a platform for the entire organization."
After: "Non-engineers use it too, mostly for dashboards."

Rule: Cut the "not just X, it's Y" construction and state the actual point directly.

---

## 10. Rule of three

Always grouping items in threes regardless of whether there are three natural items, because triplets feel rhetorically complete.

Before: "The system is fast, reliable, and scalable."
After: "The system handles 50,000 requests per second with 99.9% uptime."

Rule: List as many items as there actually are; if you have two, list two; if five, list five.

---

## 11. Synonym cycling

Rotating synonyms for the same referent within a short passage to avoid repeating a word, producing an effect worse than repetition.

Before: "The protagonist faces a difficult choice. The main character must decide quickly. The central figure weighs his options."
After: "Han faces a difficult choice. He must decide quickly."

Rule: Pick one word for one thing and use it throughout; repetition is clearer than rotation.

---

## 12. False ranges

"From X to Y" used to imply exhaustive coverage when only two arbitrary endpoints are named.

Before: "The guide covers everything from beginner setup to advanced deployment strategies."
After: "The guide covers installation and basic configuration. Advanced deployment is not included."

Rule: Replace "from X to Y" with an honest list of what is and is not covered.

---

## 13. Em dash overuse

Using an em dash where a comma, period, or colon does the same job — often two or three per paragraph — producing a staccato rhythm that signals automated drafting.

Before: "The update ships Tuesday — assuming tests pass — and includes the new API — which was delayed twice."
After: "The update ships Tuesday if tests pass. It includes the new API, which was delayed twice."

Rule: Allow one em dash per paragraph at most; replace the rest with commas, periods, or colons.

---

## 14. Boldface overuse

Bolding every noun or concept that seems important, turning emphasis into noise.

Before: "The **cache layer** sits between the **database** and the **application server**. It uses **Redis** for **fast** lookups."
After: "The cache layer sits between the database and the application server. It uses Redis."

Rule: Bold only one thing per section: the term being defined, or the one fact the reader must not miss.

---

## 15. Inline-header lists

Repeating the list item label inside the body text, as if the label alone were not enough.

Before: "Performance: Performance improved significantly after the refactor."
After: "Performance: improved significantly after the refactor."

Rule: After a bold or heading label, do not restate the label in the sentence that follows.

---

## 16. Title case headings

Capitalizing every major word in a heading, mimicking the style of formal documents regardless of context.

Before: "## The History Of The Project And Its Key Contributors"
After: "## Project history"

Rule: Use sentence case for all headings: capitalize only the first word and proper nouns.

---

## 17. Emojis as structure

Using emojis as visual bullets or section markers, which degrades to symbols when rendered in plain text and signals informality regardless of context.

Before: "🚀 Launch Phase: 💡 Key Insight: The team moved fast."
After: "Launch phase. Key insight: the team moved fast."

Rule: Remove all emojis used as bullets, headers, or decorators; use plain punctuation and whitespace instead.

---

## 18. Curly/smart quotes

Using typographic quotes (" " ' ') instead of straight ASCII quotes (" ') in plain text, code, or configuration files.

Before: "He said 'the build is broken.'"
After: "He said 'the build is broken.'"

Rule: Use straight quotes (" ') everywhere; configure your editor to not autocorrect them in prose intended for programmatic processing.

---

## 19. Chatbot artifacts

Conversational sign-off phrases carried over from dialogue-tuned models into static written content.

Before: "I hope this helps! Feel free to let me know if you have any other questions or need further clarification."
After: [nothing — end the document where the content ends]

Rule: Delete every sentence that exists only to close a conversation; documents do not say goodbye.

---

## 20. Cutoff disclaimers

Hedges that disclose training-data staleness, meaningful in a chat session, meaningless in a document.

Before: "While details are limited in available sources, as of my last update this approach was considered best practice."
After: "This was considered best practice in 2023. Check the current RFC."

Rule: Replace cutoff disclaimers with a specific date and a pointer to the authoritative current source.

---

## 21. Sycophantic tone

Openers that praise the question or affirm the reader before answering.

Before: "Great question! You're absolutely right that this is a complex area. Certainly, I'd be happy to explain."
After: [start with the answer]

Rule: Delete the first sentence of any paragraph that contains "great", "absolutely", "certainly", or "happy to".

---

## 22. Filler phrases

Multi-word constructions that add length without meaning.

Phrases to cut: "in order to", "due to the fact that", "it is worth noting that", "it is important to note that", "at the end of the day", "with that said".

Before: "It is important to note that in order to deploy, you need to run the migration first."
After: "Run the migration before deploying."

Rule: Replace each phrase with nothing or with the word it's standing in for ("to", "because", "note:").

---

## 23. Excessive hedging

Stacking multiple hedging modals on a single claim until it says nothing.

Before: "This approach could potentially possibly be considered an improvement in some cases."
After: "This is faster in write-heavy workloads. It may be slower for reads."

Rule: Allow one hedge per claim ("could", "might", "often"); cut every additional one and make the conditions explicit instead.

---

## 24. Generic conclusions

Closing paragraphs that summarize nothing and predict a bright future, present in almost every LLM-generated document regardless of topic.

Before: "The future looks bright for this technology. Exciting times lie ahead as the ecosystem continues to grow and evolve."
After: [end on the last substantive point]

Rule: Delete any final paragraph whose sentences would be equally true of any other topic.
