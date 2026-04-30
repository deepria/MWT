import type { AuthUser } from '@/stores/auth'

interface TokenResponse {
  id_token: string
  access_token: string
  refresh_token?: string
  expires_in: number
  token_type: string
}

const clientId = import.meta.env.VITE_COGNITO_CLIENT_ID
const domain = normalizeDomain(import.meta.env.VITE_COGNITO_DOMAIN)
const redirectSignIn =
  import.meta.env.VITE_COGNITO_REDIRECT_SIGN_IN ??
  `${window.location.origin}/auth/callback`
const redirectSignOut =
  import.meta.env.VITE_COGNITO_REDIRECT_SIGN_OUT ??
  `${window.location.origin}/login`
const scopes = import.meta.env.VITE_COGNITO_SCOPES ?? 'openid email'

const PKCE_VERIFIER_KEY = 'mwt.auth.pkce.verifier'
const OAUTH_STATE_KEY = 'mwt.auth.oauth.state'
const TOKEN_KEY = 'mwt.auth.tokens'

function normalizeDomain(value: string | undefined) {
  if (!value) return ''
  return value.replace(/\/$/, '')
}

function getHostedUiConfig() {
  if (!clientId || !domain) {
    throw new Error(
      'Hosted UI 설정이 비어 있습니다. VITE_COGNITO_CLIENT_ID와 VITE_COGNITO_DOMAIN을 확인하세요.',
    )
  }

  return {
    clientId,
    domain,
  }
}

function base64UrlEncode(bytes: ArrayBuffer) {
  const binary = String.fromCharCode(...new Uint8Array(bytes))

  return window
    .btoa(binary)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '')
}

function randomBase64Url(byteLength: number) {
  const bytes = new Uint8Array(byteLength)
  window.crypto.getRandomValues(bytes)

  return base64UrlEncode(bytes.buffer)
}

async function sha256(value: string) {
  const bytes = new TextEncoder().encode(value)
  return window.crypto.subtle.digest('SHA-256', bytes)
}

function decodeJwtPayload(token: string): Record<string, unknown> {
  const payload = token.split('.')[1]
  const normalized = payload.replace(/-/g, '+').replace(/_/g, '/')
  const padded = normalized.padEnd(
    normalized.length + ((4 - (normalized.length % 4)) % 4),
    '=',
  )

  return JSON.parse(window.atob(padded)) as Record<string, unknown>
}

function tokenToUser(idToken: string): AuthUser {
  const payload = decodeJwtPayload(idToken)
  const groups = payload['cognito:groups']
  const fallbackEmail = String(payload['cognito:username'] ?? payload.sub)

  return {
    id: String(payload.sub ?? fallbackEmail),
    email: String(payload.email ?? fallbackEmail),
    groups: Array.isArray(groups) ? groups.map(String) : ['participant'],
  }
}

export async function redirectToHostedUi(redirectPath = '/problems') {
  const config = getHostedUiConfig()

  const verifier = randomBase64Url(64)
  const challenge = base64UrlEncode(await sha256(verifier))
  const state = randomBase64Url(32)

  window.sessionStorage.setItem(PKCE_VERIFIER_KEY, verifier)
  window.sessionStorage.setItem(
    OAUTH_STATE_KEY,
    JSON.stringify({ state, redirectPath }),
  )

  const params = new URLSearchParams({
    client_id: config.clientId,
    code_challenge: challenge,
    code_challenge_method: 'S256',
    redirect_uri: redirectSignIn,
    response_type: 'code',
    scope: scopes,
    state,
  })

  window.location.assign(
    `${config.domain}/oauth2/authorize?${params.toString()}`,
  )
}

export async function completeHostedUiLogin(
  code: string,
  returnedState: string,
): Promise<{ user: AuthUser; redirectPath: string }> {
  const config = getHostedUiConfig()

  const verifier = window.sessionStorage.getItem(PKCE_VERIFIER_KEY)
  const storedState = window.sessionStorage.getItem(OAUTH_STATE_KEY)

  if (!verifier || !storedState) {
    throw new Error('로그인 세션 정보가 없습니다. 다시 로그인하세요.')
  }

  const parsedState = JSON.parse(storedState) as {
    state: string
    redirectPath: string
  }

  if (parsedState.state !== returnedState) {
    throw new Error('로그인 state가 일치하지 않습니다. 다시 로그인하세요.')
  }

  const body = new URLSearchParams({
    client_id: config.clientId,
    code,
    code_verifier: verifier,
    grant_type: 'authorization_code',
    redirect_uri: redirectSignIn,
  })

  const response = await window.fetch(`${config.domain}/oauth2/token`, {
    method: 'POST',
    headers: {
      'content-type': 'application/x-www-form-urlencoded',
    },
    body,
  })

  if (!response.ok) {
    throw new Error(
      'Cognito token 교환에 실패했습니다. Hosted UI 설정을 확인하세요.',
    )
  }

  const tokenResponse = (await response.json()) as TokenResponse
  const user = tokenToUser(tokenResponse.id_token)

  window.localStorage.setItem(TOKEN_KEY, JSON.stringify(tokenResponse))
  window.sessionStorage.removeItem(PKCE_VERIFIER_KEY)
  window.sessionStorage.removeItem(OAUTH_STATE_KEY)

  return {
    user,
    redirectPath: parsedState.redirectPath,
  }
}

export function redirectToHostedUiLogout() {
  if (!domain || !clientId) return

  const params = new URLSearchParams({
    client_id: clientId,
    logout_uri: redirectSignOut,
  })

  window.location.assign(`${domain}/logout?${params.toString()}`)
}

export function clearHostedUiTokens() {
  window.localStorage.removeItem(TOKEN_KEY)
}

export function getHostedUiAuthorizationToken() {
  const raw = window.localStorage.getItem(TOKEN_KEY)

  if (!raw) return null

  try {
    const tokens = JSON.parse(raw) as Partial<TokenResponse>

    return tokens.id_token ?? tokens.access_token ?? null
  } catch {
    window.localStorage.removeItem(TOKEN_KEY)
    return null
  }
}

export function getHostedUiDebugConfig() {
  return {
    clientId: clientId ?? '',
    domain,
    scopes,
    redirectSignIn,
    redirectSignOut,
  }
}
