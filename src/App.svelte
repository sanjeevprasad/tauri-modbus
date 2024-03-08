<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import Grid from './grid.svelte'
  let keydown = (e: KeyboardEvent) => {
    if (e.ctrlKey && e.code == 'KeyR') {
      location.href = location.href
    }
  }
  onMount(() => {
    document.addEventListener('keydown', keydown)
  })
  onDestroy(() => {
    document.removeEventListener('keydown', keydown)
  })
</script>

<div class="flex flex-col h-full">
  <Grid />
</div>
