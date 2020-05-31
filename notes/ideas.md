# Ideas for stuff to build.

This is essentially a wishlist of features to attempt.

Rough outline of things to tackle might be:

- render some text.
- paginated text.
- graphics (bg, "speaker portrait").
- work out system for text "effects".
- menu system for user "speech" (ie, select a dialog option).
- system for branching npc dialogue based on player choices.

### Gradual Text Rendering

- gradual text - render character by character at a certain crawl speed.
- speed up the crawl speed via user input.
- zip to the end, again by user input.

### Text Pagination

- text will be rendered in pages.
- wait for user input to advance to the next page.

### Speaker System

- system for setting the "speaker" for dialogue.
- the speaker informs how text treatments are rendered (ie a speaker brings
  their own set of effects/colors for each **sentiment**).
- speakers have their own artwork to show as a portrait.

### Speaker Portrait Control

Seems like we want to have some control over the artwork for dramatic effect.

([thread](https://twitter.com/workingjubilee/status/1265823777151057921))

- provide a way to enable/disable the speaker portrait.
- perhaps have a way to cue different artwork selections given control codes in
  the dialogue text.
- portrait *effects* like a camera shake, or decals/overlays.

### Conversation System

- Allow user to select from a menu of dialogue options.
- Branch npc dialogue based on user choices.


### Audio Cues

Would be great to get audio to play, triggered by markers in the dialogue text.

### Branch Configuration

It would be good to have a way to open/close certain branches in a sequence
based on factors external to the dialogue text itself, such as general
*game state*.

It's possible this could be solved by splitting the sequence up into smaller
parts and using actual game state transitions to mix and match arbitrarily.
Another idea might be to offer a hybrid approach where a series of passages
could each get tags associated with them, and the caller code could "filter" the
passages using those tags. 