export interface FollowingAccount {
  id: string;
  screen_name: string;
  name: string;
  description?: string | null;
  followers_count?: number;
  friends_count?: number;
  statuses_count?: number;
  favourites_count?: number;
  listed_count?: number;
  location?: string | null;
  website?: string | null;
  verified_type?: string | null;
  is_blue_verified?: boolean;
  following?: boolean;
  protected?: boolean;
  created_at?: string;
  url?: string;
  profile_image_url?: string;
  profile_banner_url?: string;
  [key: string]: unknown;
}

export interface ArchivedTweet {
  id: string;
  content: string;
  timestamp: string;
  url: string;
  author: string;
}

export interface ArchiveMeta {
  capturedAt: string;
  tweetLimit: number;
  tweetsFetched: number;
  pagesFetched: number;
  sourceFile: string;
  errors?: string[];
  durationMs?: number;
  cursorTrail?: string[];
  attempts: number;
  skipped?: boolean;
}

export interface AccountArchive {
  account: FollowingAccount;
  tweets: ArchivedTweet[];
  meta: ArchiveMeta;
}
