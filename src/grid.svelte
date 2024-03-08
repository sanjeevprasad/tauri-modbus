<script lang="ts">
  import { invoke } from '@tauri-apps/api'
  import { EventCallback, listen, UnlistenFn } from '@tauri-apps/api/event'
  import { onDestroy, onMount } from 'svelte'
  interface IData {
    msg: string
    data: number
  }
  let data: IData[] = new Array(1000).fill(0).map((_e, i) => ({
    data: 0,
    msg: '',
  }))
  let serial_port = 0
  let port = ''
  let ports: string[] = []
  let busy = false
  let baud_rate
  let keep_connecting = false
  let connected = false
  let state_err = ''
  let onChange = async (index: number) => {
    let result = await invoke('set_modbus_reg', { index, value: data[index] })
    console.log('set_modbus_reg', result)
  }
  let connect = async () => {
    await invoke<boolean>('set_keep_connecting', {
      value: true,
    }).catch((err) => {
      console.error(err)
      state_err = err
      return false
    })
  }
  let logs: string[] = [`INFO: Logs enabled`]
  let onLog: EventCallback<string> = (event) => {
    let log = event.payload
    logs.push(log)
    console.log(log)
    logs = logs
  }
  let unlisten: UnlistenFn[] = []
  onMount(async () => {
    unlisten = [
      await listen<string>('log', onLog), //
      await listen<number>('baud_rate', ({ payload }) => (baud_rate = payload)),
      await listen<number>('serial_port', ({ payload }) => (serial_port = payload)),
      await listen<boolean>('keep_connecting', ({ payload }) => (keep_connecting = payload)),
      await listen<boolean>('connected', ({ payload }) => (connected = payload)),
      await listen<string[]>('available_ports', ({ payload }) => (ports = payload)),
    ]
    await invoke('refresh_state')
    await invoke('refresh_ports')
  })
  onDestroy(() => {
    unlisten.forEach((u) => u && u())
  })
</script>

<div class="flex flex-row h-full">
  <div class="flex flex-col h-full w-7/12 overflow-y-auto">
    <div class="flex flex-col p-1">
      <div class="flex flex-row p-1">
        <div class="flex p-1">Port:</div>
        <select class="flex p-1" bind:value={port} disabled={busy}>
          <option value="" hidden>--SELECT--</option>
          {#each ports as p}
            <option value={p}>{p}</option>
          {/each}
        </select>
      </div>
      <div class="flex p-1">
        {#if busy}
          <button
            class="btn py-1 rounded"
            class:btn-success={connected}
            class:btn-secondary={!connected}
            on:click={connect}
          >
            {connected ? 'connected' : '...'}
          </button>
        {:else}
          <button class="btn py-1 rounded" class:btn-secondary={true} on:click={connect}>
            Connect
          </button>
        {/if}
      </div>
      <div class="flex p-1">
        <div class="text-red-500">{state_err}</div>
      </div>
    </div>
    <div class="flex">
      <table class="border-collapse">
        <tr>
          <th class="border bg-slate-100">Address</th>
          <th class="border bg-slate-100">Value</th>
        </tr>
        {#each data as d, i}
          <tr>
            <td class="border bg-slate-100 text-right font-mono">{i}</td>
            <td class="border">
              <input
                class="text-right"
                type="number"
                bind:value={d.data}
                on:change={() => onChange(i)}
              />
            </td>
          </tr>
        {/each}
      </table>
    </div>
  </div>
  <div class="flex flex-col border-gray-600 border-l w-5/12 h-full overflow-y-auto">
    {#each logs as l}
      <pre
        class="border-gray-300 border-b"
        class:bg-gray-100={l[0] == 'I'}
        class:bg-red-100={l[0] == 'E'}>{l}</pre>
    {/each}
  </div>
</div>
