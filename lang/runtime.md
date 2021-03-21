# Goals

- Can easily deploy a system to a new environment
- System can run on dev box, cluster of servers, or end user box (ala web browser)
- Runtime is completely inspectible (ala smalltalk)
- Runtime is a complete solution for user interaction (GUI, console)
- Supports uncancellable and generally secure relationships between end users & developers
  - No need for users to register with email or phone, but rather a unique token tied to that specific bidirectional relationship
  - Failure to protect that token means an attacker can impersonate the user or service, so sharing should be minimized
  - This relationship allows the delivery of messages between user & service and AuthN/Z 
  - This same mechanism should be used to facilitate trust relationships between dev environments -> QA env -> Prod env and so on
- Runtime has complete support for unit & integration testing
  

# What is contained in an Environment?

An environment is specified by a set of Newt source files, persistent state (if any), an identity, and trust relationships with other environments.

On a devbox, this can be represented directly in the filesystem under some root folder with some special conventions.  
Perhaps a structure like the following would suffice for an environment named 'dev':

- /dev.state
- /dev.identity
- /relationships/qa.identity
- /children/{child_name}/**
- /children/{child_name}/src_{unique_slug_per_version}/
- /src/**

The identity files should be textual, and the relevant QA environment should have a reciprocal identity file.  The state file should probably be binary.

In order to support deployment of an environment into different contexts (dev box, server cluster, client box) we need to support
nested environments.  The dev box parent environment might be a Smalltalk-esque IDE, while the server cluster might be an agent which 
manages the health of the cluster -- scaling up & down, emitting telemetry etc, and the client box is similar to a web browser.

When a deployment of a child environment happens, no changes are made to the parent environments.  The parent environments have complete control over how their children interact with the outside world.  In these cases the parent environment manages the transition
between versions of the child environment.  For a web service this might mean spinning up the new version, directing traffic to it, and spinning down the old version when it finishes processing it's in-flight requests.

# How is an environment deployed? Dev -> QA

The developer is happy with their latest changes and wants to deploy them from his box to QA, a cluster of servers.
Upon issuing the appropriate command, his Dev environment connects to a node in the QA environment.  It then uploads the relevant folder structure to that node.  The QA node then forwards those updates to it's siblings.  At which point each node begins switching traffic over to the new version.
