<script lang="ts">
    export let settingsOpen = false;
    export let toggleSettings: () => void;

    let hotkeyConfig = '';
    let promptFilePath = '';
    let themeSelection: 'light' | 'dark' = 'dark'; // По умолчанию темная тема

    function closeSettings() {
        toggleSettings();
    }

    function saveSettings() {
        // Логика для сохранения настроек
        console.log('Settings saved');
        toggleSettings();
    }

    function changeTheme(event: Event) {
        const value = (event.target as HTMLSelectElement).value;

        if (value === "light" || value === "dark") {
            themeSelection = value;

            if (value === "dark") {
                document.body.classList.add("dark-theme");
                document.body.classList.remove("light-theme");
            } else {
                document.body.classList.add("light-theme");
                document.body.classList.remove("dark-theme");
            }
        } else {
            console.error("Некорректное значение темы:", value);
        }
    }
</script>

{#if settingsOpen}
    <div id="settings-modal" class="settings-modal">
        <div class="settings-modal-content">
            <div class="modal-header">
                <h2>Настройки</h2>
                <button
                        id="close-settings-btn"
                        class="close-settings-btn"
                        aria-label="Закрыть"
                        on:click={closeSettings}
                >
                    <i class="fas fa-times"></i>
                </button>
            </div>
            <div class="settings-form">
                <div class="form-group">
                    <label for="prompt-file-path">Путь к файлу промптов:</label>
                    <div class="file-input-group">
                        <input
                                id="prompt-file-path"
                                type="text"
                                bind:value={promptFilePath}
                                placeholder="Выберите файл с промптами"
                        />
                        <button
                                id="browse-btn"
                                class="browse-button"
                                aria-label="Обзор"
                        >
                            <i class="fas fa-folder-open"></i>
                        </button>
                    </div>
                </div>
                <div class="form-group">
                    <label for="hotkey-config">Горячая клавиша:</label>
                    <input
                            id="hotkey-config"
                            type="text"
                            bind:value={hotkeyConfig}
                            placeholder="Например: Ctrl+Space"
                    />
                </div>
                <div class="form-group">
                    <label for="theme-selection">Выберите тему:</label>
                    <select id="theme-selection" bind:value={themeSelection} on:change={changeTheme}>
                        <option value="dark">Темная</option>
                        <option value="light">Светлая</option>
                    </select>
                </div>
                <button
                        id="save-settings-btn"
                        class="action-button"
                        on:click={saveSettings}
                >
                    Сохранить
                </button>
            </div>
        </div>
    </div>
{/if}
