# Contributing to Manta Network

:+1::tada: First off, thanks for taking the time to contribute! :tada::+1:

The following is a set of guidelines for contributing to the Manta Network codebase. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## How Can I Contribute?

### Reporting Bugs

Following these guidelines helps maintainers and the community understand your report :pencil:, reproduce the behavior :computer: :computer:, and find related reports :mag_right:.

* **Perform a cursory search in the Manta Network organization to see if the problem has already been reported. If it has **and the issue is still open**, add a comment to the existing issue instead of opening a new one.

> **Note:** If you find a **Closed** issue that seems like it is the same thing that you're experiencing, open a new issue and include a link to the original issue in the body of your new one.

#### How Do I Submit A (Good) Bug Report?

Bugs are tracked as [GitHub issues](https://guides.github.com/features/issues/). 

Explain the problem and include additional details to help maintainers reproduce the problem:

* **Use a clear and descriptive title** for the issue to identify the problem.
* **Describe the exact network configuration** include parachain node version, relay chain node version and any tooling you may be using, such as polkadot-launch and its configurations.
* **Describe the exact steps which reproduce the problem** in as many details as possible. For example, start by explaining how you started the Manta node, e.g. which command exactly you used in the terminal, how you started the relay chain, etc.
* **Provide specific examples to demonstrate the steps**. Include links to files or GitHub projects, or copy/pasteable snippets, which you use in those examples. If you're providing snippets in the issue, use [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
* **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
* **Explain which behavior you expected to see instead and why.**
* **Include screenshots or video** if possible.
* **If the problem is related to performance or memory**, include a CPU profile capture.
* **If the problem wasn't triggered by a specific action**, describe what you were doing before the problem happened.

Furthermore, provide more context by answering these questions:

* **Did the problem start happening recently** (e.g. after updating to a new version of Manta/Calamari) or was this always a problem?
* If the problem started happening recently, **can you reproduce the problem in an older version of Manta/Calamari?** What's the most recent version in which the problem doesn't happen? You can download older versions from [the releases page](https://github.com/Manta-Network/Manta/releases).
* **Can you reliably reproduce the issue?** If not, provide details about how often the problem happens and under which conditions it normally happens.

### Always include details about your configuration and environment

* **Are you working with the Manta/Calamari mainnets or have you deployed locally**
* **Which version of Manta/Calamari are you using?**
* **What relay chain and version are you using?**
* **What's the name and version of the OS you're using**?
* **Are you running Manta/Calamari with Docker?** 
* **Did you compile the code on your own?** and if so, what compiler and version did you use?

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for Manta/Calamari, including completely new features and minor improvements to existing functionality. Following these guidelines helps maintainers and the community understand your suggestion :pencil: and find related suggestions :mag_right:.

Before creating enhancement suggestions, please check the existing issues as you might find out that you don't need to create one. When you are creating an enhancement suggestion, please [include as many details as possible](#how-do-i-submit-a-good-enhancement-suggestion).  including the steps that you imagine you would take if the feature you're requesting existed.

#### How Do I Submit A (Good) Enhancement Suggestion?

Enhancement suggestions are tracked as [GitHub issues](https://guides.github.com/features/issues/). Create an issue on that repository and provide the following information:

* **Use a clear and descriptive title** for the issue to identify the suggestion.
* **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
* **Provide specific examples to demonstrate the steps**. Include copy/pasteable snippets which you use in those examples, as [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
* **Describe the current behavior** and **explain which behavior you expected to see instead** and why.
* **Include screenshots or video** which help you demonstrate the steps or point out the part of Manta/Calamari which the suggestion is related to.
* **Explain why this enhancement would be useful** to most Manta users.
* **List some other places where this enhancement exists.**

### Pull Requests

The process described here has several goals:

- Maintain code quality
- Fix problems that are important to users
- Engage the community in working toward the best possible Manta
- Enable a sustainable system for Manta's maintainers to review contributions

Please follow these steps to have your contribution considered by the maintainers:

1. Every commit needs to be signed-off with your name, email and [gpg key](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits):
    1.1. `git config --global user.name your_name`
    1.2. `git config --global user.email you_email`
    1.3. `git commit -s -S -m your_commit_message`
        * `-s` = `Signed-off-by`
        * `-S` = `Verify commit using gpg key`
2. Follow all instructions in [the template](.github/PULL_REQUEST_TEMPLATE.md)
3. After you submit your pull request, verify that all CI checks are passing <details><summary>What if CI checks are failing?</summary>If a CI check is failing, and you believe that the failure is unrelated to your change, please leave a comment on the pull request explaining why you believe the failure is unrelated. A maintainer will re-run the status check for you. If we conclude that the failure was a false positive, then we will open an issue to track that problem.</details>
4. If you have a significant design decision to make please document it as an ehancement request first, so we can review it and provide feedback before any code is written.

While the prerequisites above should be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional design work, tests, or other changes before your pull request can be ultimately accepted.

## Styleguides

* We generally follow the [Rust styleguide](https://doc.rust-lang.org/1.0.0/style/).

### Comments

* Comments should focus more on the `why` instead of the `what` is the code.
* Write comments on top of your code.

### PR descriptions

* Descriptions should link to issues with the keyword `closes #issue_number`
* Descriptions should include a list of the new changes and why they were made.
* Descriptions should include links to other relevant issues/pull requests.

### Git Commit Messages

* Write clear and concise commit messages.
* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Don't forget to sign your commits.
