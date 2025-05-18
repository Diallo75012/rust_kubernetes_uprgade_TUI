# Kubernetes Upgrade TUI Manager
- This project is done after having upgraded manually my Kubernetes cluster twice so I had created a bash script to handle that but as I am learning Rust
  I found here an amazing occasion to make something nice, a `TUI` with less dependencies as possible (for my level of coding) and I will learn by trying
  and making mini modules along the way and have `ChatGPT` as my `Senior Enginer Not GateKeeper` guidng me along the way.

# App Desired Flow:
- user enters a intermediary version or Kuberneter like `1.<...>` in the `TUI` and then the backend would do the full upgrade of the cluster
  - cordon/uncordon
  - pull key/repo
  - upgrade `kubect`/`kueadm`/`kubelet` and optionally `containerD` (as it doesn't need updates as frequently as kubernetes does)
  - parsers, checkers of version
  - `TUI` having all steps listed. And changing color of steps when validated and done. And it is `sequential` steps.
  - `TUI` will display all the time the versions of `kubectl`, `kubelet`, `kubeadm` and `containrd` and change those colors if they changed from start version avaialble
  - `TUI` would be simple but dynamic, focusing on each steps. thereofore, streaming `stdout`/`stderr` to display what is going on
  - `TUI` won't put notification service to user to make it easy willjust change the screen states to say if it successful or stop with red color if it is failed
  - `TUI` steps of upgrade will be greyed out and become `green` when validated and done, `orange` when being performed, `red` which means error so stop at that step
  - `TUI` when it is `red` step and stops, will display error message. User can only re-enter the version to restart all, or user need to contact `devs` to fix app


## Tips
- Do restart `kubelet` and `containerd` before starting the app to be sure to have the cluster available during `upgrade plan` and `upgrade apply` steps.
- You need to go and find the right versions of `kube` components which is assumed that they all have same version and `containerd` compatible one.
  - kube component, just enter minor version: `1.29` for example
  - containerd, enter the full version: `1.7.25-1` for example
  After the app will search for the versionsa nd parse what is needed for each steps
