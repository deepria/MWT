<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

const dino_el = ref<HTMLDivElement | null>(null)
const cactus_el = ref<HTMLDivElement | null>(null)

const message = ref('PRESS SPACE TO START')
const score = ref(0)
const high_score = ref(0)
const is_running = ref(false)
const is_jumping = ref(false)

const score_text = computed(() => String(score.value).padStart(5, '0'))
const high_score_text = computed(() =>
  String(high_score.value).padStart(5, '0'),
)

let dino_y = 0
let velocity_y = 0
let cactus_x = 720
let speed = 6
let animation_id = 0

const gravity = 0.8
const jump_power = 15
const ground_y = 0

function start_game() {
  if (is_running.value) {
    jump()
    return
  }

  is_running.value = true
  is_jumping.value = false
  dino_y = 0
  velocity_y = 0
  cactus_x = 720
  score.value = 0
  speed = 6
  message.value = ''

  cancelAnimationFrame(animation_id)
  game_loop()
}

function jump() {
  if (!is_running.value || is_jumping.value) return

  is_jumping.value = true
  velocity_y = jump_power
}

function game_loop() {
  update_dino()
  update_cactus()
  update_score()

  if (is_collision()) {
    end_game()
    return
  }

  animation_id = requestAnimationFrame(game_loop)
}

function update_dino() {
  if (!dino_el.value) return

  if (is_jumping.value) {
    dino_y += velocity_y
    velocity_y -= gravity

    if (dino_y <= ground_y) {
      dino_y = ground_y
      velocity_y = 0
      is_jumping.value = false
    }
  }

  dino_el.value.style.bottom = `${47 + dino_y}px`
}

function update_cactus() {
  if (!cactus_el.value) return

  cactus_x -= speed

  if (cactus_x < -60) {
    cactus_x = 760 + Math.random() * 240
    speed += 0.25
  }

  cactus_el.value.style.left = `${cactus_x}px`
}

function update_score() {
  score.value = Math.floor(score.value + 1)
}

function is_collision() {
  if (!dino_el.value || !cactus_el.value) return false

  const dino_rect = dino_el.value.getBoundingClientRect()
  const cactus_rect = cactus_el.value.getBoundingClientRect()
  const padding = 8

  return !(
    dino_rect.right - padding < cactus_rect.left + padding ||
    dino_rect.left + padding > cactus_rect.right - padding ||
    dino_rect.bottom - padding < cactus_rect.top + padding ||
    dino_rect.top + padding > cactus_rect.bottom - padding
  )
}

function end_game() {
  is_running.value = false
  high_score.value = Math.max(high_score.value, score.value)
  message.value = 'GAME OVER'
  cancelAnimationFrame(animation_id)
}

function handle_keydown(event: KeyboardEvent) {
  if (event.code === 'Space' || event.code === 'ArrowUp') {
    event.preventDefault()
    if (is_running.value) {
      jump()
    } else {
      start_game()
    }
  }
}

onMounted(() => {
  document.addEventListener('keydown', handle_keydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handle_keydown)
  cancelAnimationFrame(animation_id)
})
</script>

<template>
  <section class="content-stack index-view">
    <div class="page-heading">
      <p class="eyebrow">MWT Online Judge</p>
      <h1>환영합니다.</h1>
      <p>문제를 풀어볼까요?</p>
    </div>

    <div id="game" class="game">
      <div class="score-board">
        <span>HI {{ high_score_text }}</span>
        <span>{{ score_text }}</span>
      </div>
      <div class="message">{{ message }}</div>
      <div
        ref="dino_el"
        class="dino"
        :class="{ running: is_running && !is_jumping }"
      ></div>
      <div ref="cactus_el" class="cactus"></div>
      <div class="ground"></div>
    </div>
  </section>
</template>

<style scoped>
.index-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #333;
}

.game {
  position: relative;
  width: 720px;
  height: 260px;
  overflow: hidden;
  border: none;
  font-family: monospace;
}

.score-board {
  position: absolute;
  top: 20px;
  right: 24px;
  z-index: 2;
  display: flex;
  gap: 18px;
  font-size: 18px;
  font-weight: bold;
  color: #535353;
}

.message {
  position: absolute;
  top: 72px;
  z-index: 2;
  width: 100%;
  text-align: center;
  font-size: 18px;
  font-weight: bold;
  color: #535353;
}

.ground {
  position: absolute;
  left: 0;
  bottom: 44px;
  width: 200%;
  height: 2px;
  background: repeating-linear-gradient(
    to right,
    #535353 0 60px,
    transparent 60px 72px,
    #535353 72px 130px,
    transparent 130px 150px
  );
  animation: ground-move 0.8s linear infinite;
}

@keyframes ground-move {
  from {
    transform: translateX(0);
  }

  to {
    transform: translateX(-360px);
  }
}

.dino {
  position: absolute;
  left: 70px;
  bottom: 47px;
  width: 44px;
  height: 48px;
  background: #535353;
  clip-path: polygon(
    20% 0,
    70% 0,
    70% 15%,
    100% 15%,
    100% 45%,
    75% 45%,
    75% 65%,
    60% 65%,
    60% 100%,
    48% 100%,
    48% 75%,
    35% 75%,
    35% 100%,
    23% 100%,
    23% 70%,
    0 70%,
    0 45%,
    20% 45%
  );
}

.dino.running::before {
  content: '';
  position: absolute;
  left: 12px;
  bottom: -8px;
  width: 8px;
  height: 12px;
  background: #535353;
  animation: leg-left 0.18s infinite alternate;
}

.dino.running::after {
  content: '';
  position: absolute;
  left: 28px;
  bottom: -8px;
  width: 8px;
  height: 12px;
  background: #535353;
  animation: leg-right 0.18s infinite alternate;
}

@keyframes leg-left {
  from {
    transform: translateY(0);
  }

  to {
    transform: translateY(8px);
  }
}

@keyframes leg-right {
  from {
    transform: translateY(8px);
  }

  to {
    transform: translateY(0);
  }
}

.cactus {
  position: absolute;
  left: 720px;
  bottom: 47px;
  width: 26px;
  height: 52px;
  background: #535353;
  border-radius: 2px 2px 0 0;
}

.cactus::before,
.cactus::after {
  content: '';
  position: absolute;
  width: 10px;
  height: 26px;
  background: #535353;
}

.cactus::before {
  left: -10px;
  top: 18px;
}

.cactus::after {
  right: -10px;
  top: 10px;
}
</style>
