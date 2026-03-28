import {
  getAuthType,
  type ApiKeyLocation,
  type AuthConfig,
  type AuthType,
  type OAuth2Config,
  type OAuthGrantType,
} from "../types";

/** Fields representing the full auth editor state */
export interface AuthFields {
  authType: AuthType;
  bearerToken: string;
  basicUsername: string;
  basicPassword: string;
  apiKeyKey: string;
  apiKeyValue: string;
  apiKeyLocation: ApiKeyLocation;
  oauthGrantType: OAuthGrantType;
  oauthClientId: string;
  oauthClientSecret: string;
  oauthAuthUrl: string;
  oauthTokenUrl: string;
  oauthScopes: string;
  oauthAccessToken: string;
  oauthRefreshToken: string;
  oauthTokenExpiry: string | null;
}

/** Default (empty) OAuth2 field values */
export function defaultOAuth2Fields(): Pick<
  AuthFields,
  | "oauthGrantType"
  | "oauthClientId"
  | "oauthClientSecret"
  | "oauthAuthUrl"
  | "oauthTokenUrl"
  | "oauthScopes"
  | "oauthAccessToken"
  | "oauthRefreshToken"
  | "oauthTokenExpiry"
> {
  return {
    oauthGrantType: "client_credentials",
    oauthClientId: "",
    oauthClientSecret: "",
    oauthAuthUrl: "",
    oauthTokenUrl: "",
    oauthScopes: "",
    oauthAccessToken: "",
    oauthRefreshToken: "",
    oauthTokenExpiry: null,
  };
}

/** Parse an AuthConfig into flat editor fields */
export function parseAuthConfig(auth?: AuthConfig): AuthFields {
  const base: AuthFields = {
    authType: getAuthType(auth),
    bearerToken: "",
    basicUsername: "",
    basicPassword: "",
    apiKeyKey: "",
    apiKeyValue: "",
    apiKeyLocation: "header",
    ...defaultOAuth2Fields(),
  };

  if (auth && "bearerToken" in auth) {
    base.bearerToken = auth.bearerToken._0.token;
  } else if (auth && "basicAuth" in auth) {
    base.basicUsername = auth.basicAuth._0.username;
    base.basicPassword = auth.basicAuth._0.password;
  } else if (auth && "apiKey" in auth) {
    base.apiKeyKey = auth.apiKey._0.key;
    base.apiKeyValue = auth.apiKey._0.value;
    base.apiKeyLocation = auth.apiKey._0.location;
  } else if (auth && "oauth2" in auth) {
    const o = auth.oauth2;
    base.oauthGrantType = o.grantType;
    base.oauthClientId = o.clientId;
    base.oauthClientSecret = o.clientSecret;
    base.oauthAuthUrl = o.authUrl;
    base.oauthTokenUrl = o.tokenUrl;
    base.oauthScopes = o.scopes;
    base.oauthAccessToken = o.accessToken;
    base.oauthRefreshToken = o.refreshToken;
    base.oauthTokenExpiry = o.tokenExpiry;
  }

  return base;
}

/** Build an OAuth2Config from flat fields */
export function buildOAuth2ConfigFromFields(fields: {
  oauthGrantType: OAuthGrantType;
  oauthClientId: string;
  oauthClientSecret: string;
  oauthAuthUrl: string;
  oauthTokenUrl: string;
  oauthScopes: string;
  oauthAccessToken: string;
  oauthRefreshToken: string;
  oauthTokenExpiry: string | null;
}): OAuth2Config {
  return {
    grantType: fields.oauthGrantType,
    clientId: fields.oauthClientId,
    clientSecret: fields.oauthClientSecret,
    authUrl: fields.oauthAuthUrl,
    tokenUrl: fields.oauthTokenUrl,
    scopes: fields.oauthScopes,
    accessToken: fields.oauthAccessToken,
    refreshToken: fields.oauthRefreshToken,
    tokenExpiry: fields.oauthTokenExpiry,
  };
}
