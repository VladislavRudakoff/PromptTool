<!-- App.svelte -->
<script lang="ts">
    import SearchBar from './SearchBar.svelte';
    import SettingsModal from './SettingsModal.svelte';

    let settingsOpen = false;

    function toggleSettings() {
        settingsOpen = !settingsOpen;
    }
</script>

<div id="app">
    <SearchBar on:toggleSettings={toggleSettings} />
    <SettingsModal settingsOpen={settingsOpen} toggleSettings={toggleSettings} />
</div>
