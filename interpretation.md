# Interpretation of the Product Memo and C_SQD.pdf

## Purpose

This document reconciles the product memo with `C_SQD.pdf` and establishes a working interpretation for product and implementation decisions.

The product memo supersedes `C_SQD.pdf` wherever the two documents conflict or leave an ambiguity, with one exception: revenue framing should default to `C_SQD.pdf`.

## Precedence Rule

1. If the product memo and `C_SQD.pdf` agree, treat the shared position as authoritative.
2. If they conflict or point in different directions, follow the product memo.
3. If the conflict concerns revenue, monetization emphasis, or financial sustainability, follow `C_SQD.pdf`.
4. If neither document resolves a question, prefer the interpretation that best supports C-SQD as a web-native scholarly evaluation network.

## Core Product Interpretation

C-SQD should be understood first as review marketplace and scholarly evaluation infrastructure, not as a document reader or PDF hosting product.

The central asset is the review graph: ElementReviews, SynthesisReviews, bug bounty outcomes, reviewer histories, disagreements, challenge outcomes, endorsement records, tags, and evaluation tuples. Article display, manuscript hosting, and reading interfaces exist to support the creation, discovery, reuse, and evaluation of that graph.

## Scholarly Objects

The primary object in the product should be a scholarly object rather than a PDF or journal article.

A scholarly object may represent a manuscript, preprint, published article, report, dataset, software package, protocol, or other intellectual contribution. Reviews attach to this canonical object rather than to a particular file, URL, or publisher location.

This interpretation extends the manuscript-centered language in `C_SQD.pdf` without rejecting it. Manuscripts remain the initial and most important use case, but the product architecture should not assume that all reviewable objects are manuscripts.

## Hosting and Publisher-Controlled Content

The memo resolves ambiguity around "publishing," "uploading," and "open access" in `C_SQD.pdf`.

C-SQD may host or natively render content when it has the rights to do so, such as author-submitted manuscripts, preprints, permissively licensed works, conference submissions, datasets, code, protocols, and other authorized materials.

For copyrighted or publisher-controlled works, C-SQD should preserve the external publisher or repository as the authoritative source. In those cases, C-SQD should store metadata, canonical identifiers, links, review records, and evaluation data, while review activity occurs inside C-SQD.

Therefore, "uploaded, discovered, and reviewed on the C-SQD platform" should not be interpreted as requiring C-SQD to host unauthorized copies of publisher-controlled articles.

## Web Platform First

The primary implementation should be web-native.

Authors, reviewers, readers, institutions, and funders should be able to participate through a browser. Assignment acceptance, review submission, reviewer profiles, challenge workflows, bug bounties, payment workflows, evaluation tuples, tags, communities, and institutional reporting should all be accessible from the web platform.

A desktop application may eventually be useful, but it should be treated as a specialized productivity layer for intensive professional review work. It should synchronize with the underlying C-SQD review graph rather than define the platform.

## Review Visibility and Subscriptions

`C_SQD.pdf` says manuscripts are open access while reviews and other insights may require subscription. The memo emphasizes visibility, discoverability, citability, and reuse of the review graph.

These should be reconciled as follows:

- The existence of reviews, review metadata, evaluation summaries, reviewer records, scholarly object identities, and public signals needed for discovery should be visible enough to support network effects.
- Full review text, advanced analytics, custom community-filtered evaluations, institutional reports, and other higher-value insights may be subscription-gated.
- Public review records should remain meaningful even if some detailed views or tools require payment.
- The platform should avoid hiding so much review information that reviews cease to be discoverable, citable, or reusable.

## Revenue Framing

Revenue and financial sustainability should follow `C_SQD.pdf`.

Manuscript submission fees primarily fund review activity and should not be framed as the main source of platform profit. Financial sustainability should be understood as coming mainly from secondary revenue streams, including subscriptions, review challenge fees, bug bounty fees, verified tags, nonstandard manuscript evaluations, institutional or community services, and AI assistant products.

The product memo's claim that review creation is the economic engine should be interpreted operationally, not as a revenue-priority statement. Review activity creates the network value that enables the platform's revenue streams, but the platform's profit model should default to the framing in `C_SQD.pdf`.

## Implementation Implications

Product and engineering decisions should optimize for low-friction participation in the review graph.

The platform should prioritize:

- canonical scholarly object identity
- review assignment and submission workflows
- ElementReview and SynthesisReview records
- reviewer profiles, tags, endorsements, and public reputation
- challenge and bug bounty workflows
- evaluation tuple computation and community-filtered recomputation
- browser-accessible discovery and institutional inspection
- rights-aware handling of native versus externally controlled content

The platform should not prioritize a sophisticated document reader ahead of the core review marketplace, except where native rendering directly improves review creation, annotation, or evaluation workflows for content C-SQD is allowed to display.

## Short Form

When in doubt, build C-SQD as a web-native review graph and evaluation marketplace. Treat manuscripts and articles as important reviewable objects, not as the product itself. Use native article display when rights permit, link out when rights require it, and follow `C_SQD.pdf` for revenue framing.
