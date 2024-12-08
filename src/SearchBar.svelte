<!-- SearchBar.svelte -->
<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    const dispatch = createEventDispatcher();

    let searchQuery = '';

    function toggleSettings() {
        dispatch('toggleSettings');
    }
</script>

<div class="search-container">
    <div class="search-wrapper">
        <input
                id="search-bar"
                type="text"
                bind:value={searchQuery}
                placeholder="Поиск промптов..."
                class="search-bar"
                autofocus
        />
        <button
                id="settings-btn"
                class="settings-btn"
                aria-label="Настройки"
                on:click={toggleSettings}
        >
            <i class="fas fa-cog"></i>
        </button>
    </div>
</div>