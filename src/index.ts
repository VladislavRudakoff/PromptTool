/**
 * Основной файл TypeScript для управления пользовательским интерфейсом
 * приложения поиска и управления промптами.
 */

import { invoke } from "@tauri-apps/api/core";

/** Интерфейс для структуры промпта */
interface Prompt {
    name: string;        // Название промпта
    content: string;     // Содержимое промпта
    parameters: string[]; // Параметры, которые нужно заполнить
}

/** Интерфейс для настроек приложения */
interface Settings {
    promptFilePath: string;  // Путь к файлу с промптами
    hotkey: string;         // Горячая клавиша
}

/**
 * Класс для управления промптами и пользовательским интерфейсом
 */
class PromptManager {
    private static instance: PromptManager;
    private prompts: Prompt[] = [];
    private filteredPrompts: Prompt[] = [];
    private settings: Settings = {
        promptFilePath: "",
        hotkey: ""
    };

    // DOM элементы
    private elements = {
        searchBar: document.getElementById("search-bar") as HTMLInputElement,
        promptList: document.getElementById("prompt-list")!,
        settingsModal: document.getElementById("settings-modal")!,
        settingsBtn: document.getElementById("settings-btn")!,
        closeSettingsBtn: document.getElementById("close-settings-btn")!,
        saveSettingsBtn: document.getElementById("save-settings-btn")!,
        promptFilePathInput: document.getElementById("prompt-file-path") as HTMLInputElement,
        hotkeyConfigInput: document.getElementById("hotkey-config") as HTMLInputElement,
        browseBtn: document.getElementById("browse-btn")!
    };

    private constructor() {
        this.initializeEventListeners();
        this.loadSettings().catch(console.error);
        this.initializeTheme();

        // Добавляем обработчик клика вне приложения
        document.addEventListener('click', (event) => {
            const target = event.target as HTMLElement;
            const appContainer = document.querySelector('.app-container');
            const settingsModal = document.getElementById('settings-modal');

            // Если клик был вне приложения и не в модальном окне настроек
            if (appContainer && 
                !appContainer.contains(target) && 
                settingsModal && 
                !settingsModal.contains(target) &&
                !settingsModal.classList.contains('hidden')) {
                return; // Не сворачиваем, если открыты настройки
            }

            // Если клик был вне приложения и поисковая строка не в фокусе
            if (appContainer && 
                !appContainer.contains(target) && 
                document.activeElement !== this.elements.searchBar) {
                // Вызываем Tauri API для сворачивания окна
                invoke('minimize_window');
            }
        });
    }

    /** Получение единственного экземпляра класса */
    public static getInstance(): PromptManager {
        if (!PromptManager.instance) {
            PromptManager.instance = new PromptManager();
        }
        return PromptManager.instance;
    }

    /** Инициализация обработчиков событий */
    private initializeEventListeners(): void {
        // Поиск по вводу текста
        this.elements.searchBar.addEventListener("input", () => {
            const query = this.elements.searchBar.value.trim();
            if (query) {
                this.filterPrompts(query);
                this.elements.promptList.classList.remove("hidden");
            } else {
                this.elements.promptList.classList.add("hidden");
            }
        });

        // Открытие/закрытие настроек
        this.elements.settingsBtn.addEventListener("click", () => {
            this.elements.settingsModal.classList.remove("hidden");
        });

        this.elements.closeSettingsBtn.addEventListener("click", () => {
            this.elements.settingsModal.classList.add("hidden");
        });

        // Выбор файла промптов
        this.elements.browseBtn.addEventListener("click", async () => {
            try {
                const selected = await invoke<string>("open_prompt_file_dialog");
                if (selected) {
                    this.elements.promptFilePathInput.value = selected;
                }
            } catch (error) {
                console.error("Ошибка при выборе файла:", error);
            }
        });

        // Сохранение настроек
        this.elements.saveSettingsBtn.addEventListener("click", async () => {
            try {
                await this.saveSettings();
                this.elements.settingsModal.classList.add("hidden");
                await this.loadPrompts();
            } catch (error) {
                console.error("Ошибка при сохранении настроек:", error);
            }
        });

        // Закрытие модального окна по клику вне его
        this.elements.settingsModal.addEventListener("click", (e) => {
            if (e.target === this.elements.settingsModal) {
                this.elements.settingsModal.classList.add("hidden");
            }
        });
    }

    /** Загрузка настроек */
    private async loadSettings(): Promise<void> {
        try {
            const config = await invoke<Settings>("get_config");
            this.settings = config;
            this.elements.promptFilePathInput.value = config.promptFilePath;
            this.elements.hotkeyConfigInput.value = config.hotkey;
            await this.loadPrompts();
        } catch (error) {
            console.error("Ошибка загрузки настроек:", error);
        }
    }

    /** Сохранение настроек */
    private async saveSettings(): Promise<void> {
        const newPath = this.elements.promptFilePathInput.value;
        const newHotkey = this.elements.hotkeyConfigInput.value;

        try {
            await invoke("set_prompt_file_path", { path: newPath });
            await invoke("set_hotkey", { newHotkey });
            this.settings.promptFilePath = newPath;
            this.settings.hotkey = newHotkey;
        } catch (error) {
            console.error("Ошибка сохранения настроек:", error);
            throw error;
        }
    }

    /** Загрузка промптов */
    private async loadPrompts(): Promise<void> {
        try {
            this.prompts = await invoke<Prompt[]>("get_prompts", {
                filePath: this.settings.promptFilePath
            });
        } catch (error) {
            console.error("Ошибка загрузки промптов:", error);
            this.prompts = [];
        }
    }

    /** Фильтрация промптов */
    private filterPrompts(query: string): void {
        const searchTerm = query.toLowerCase();
        this.filteredPrompts = this.prompts.filter(prompt =>
            prompt.name.toLowerCase().includes(searchTerm) ||
            prompt.content.toLowerCase().includes(searchTerm)
        );
        this.renderPromptList();
    }

    /** Отрисовка списка промптов */
    private renderPromptList(): void {
        this.elements.promptList.innerHTML = "";
        
        if (this.filteredPrompts.length === 0) {
            const emptyMessage = document.createElement("li");
            emptyMessage.textContent = "Ничего не найдено";
            this.elements.promptList.appendChild(emptyMessage);
            return;
        }

        this.filteredPrompts.forEach(prompt => {
            const li = document.createElement("li");
            li.textContent = prompt.name;
            li.addEventListener("click", () => {
                navigator.clipboard.writeText(prompt.content).catch(console.error);
                this.elements.searchBar.value = "";
                this.elements.promptList.classList.add("hidden");
            });
            this.elements.promptList.appendChild(li);
        });
    }

    /** Работа с темами */
    private initializeTheme(): void {
        const savedTheme = localStorage.getItem("theme") || "dark";
        this.setTheme(savedTheme);

        const themeToggle = document.getElementById("theme-toggle") as HTMLSelectElement;
        if (themeToggle) {
            themeToggle.value = savedTheme;
            themeToggle.addEventListener("change", (e) => {
                const selectedTheme = (e.target as HTMLSelectElement).value;
                this.setTheme(selectedTheme);
            });
        }
    }

    private setTheme(theme: string): void {
        const root = document.documentElement;
        root.setAttribute("data-theme", theme);
        localStorage.setItem("theme", theme);
    }
}

// Инициализация приложения при загрузке DOM
document.addEventListener("DOMContentLoaded", () => {
    PromptManager.getInstance();
});
