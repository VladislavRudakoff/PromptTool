import { mount } from 'svelte';
import App from './App.svelte';

console.log('Initializing Svelte app...');

const appElement = document.getElementById('app');

if (!appElement) {
  console.error('Critical: Element with id "app" not found');
  throw new Error('Element with id "app" not found');
}

console.log('Mounting Svelte app to element:', appElement);

const app = mount(App, {target: appElement});

console.log('Svelte app mounted successfully');

export default app;