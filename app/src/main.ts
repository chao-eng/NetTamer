import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './style.css'

// Disable browser default right-click context menu across the application
document.addEventListener('contextmenu', (e) => {
  e.preventDefault()
})

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.mount('#app')
