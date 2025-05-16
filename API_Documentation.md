# API Documentation for Frontend Development

This document outlines all available API endpoints for the Community Web backend. Each section includes the endpoint URL, HTTP method, request parameters, and response structure.

## General Thread Endpoints

### Create Thread
- **URL**: `/api/threads`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:
```json
{
  "title": "Thread Title",
  "content": "Thread Content",
  "room_id": 123
}
```
- **Response**:
```json
{
  "id": 1,
  "title": "Thread Title",
  "content": "Thread Content",
  "data": [],
  "room_id": 123
}
```

### List Thread IDs
- **URL**: `/api/threads`
- **Method**: `GET`
- **Response**:
```json
[1, 2, 3]
```

### Get Thread
- **URL**: `/api/thread/:id`
- **Method**: `GET`
- **Response**:
```json
{
  "id": 1,
  "title": "Thread Title",
  "content": "Thread Content",
  "replies": [
    {
      "id": "1234567890",
      "parent_id": null,
      "content": "Reply content"
    }
  ]
}
```

### Add Comment to Thread
- **URL**: `/api/comments`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:
```json
{
  "thread_id": 1,
  "parent_id": "1234567890", // Optional, omit for top-level comments
  "content": "Comment content"
}
```
- **Response**: HTTP Status 201 (Created) on success

## Researcher Card Endpoints

### Create Researcher Card
- **URL**: `/api/researcher_card`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:
```json
{
  "name": "Researcher Name",
  "affiliation": "University Name",
  "citedby": 100,
  "email_domain": "university.edu",
  "interests": ["AI", "Machine Learning"],
  "google_scholar_publication_ids": ["pub1", "pub2"],
  "google_scholar_id": "scholar123"
}
```
- **Response**:
```json
{
  "id": 1,
  "name": "Researcher Name",
  "affiliation": "University Name",
  "citedby": 100,
  "email_domain": "university.edu",
  "interests": ["AI", "Machine Learning"],
  "google_scholar_publication_ids": ["pub1", "pub2"],
  "google_scholar_id": "scholar123"
}
```

### Get Researcher Card
- **URL**: `/api/researcher_card/:id`
- **Method**: `GET`
- **Response**:
```json
{
  "id": 1,
  "name": "Researcher Name",
  "affiliation": "University Name",
  "citedby": 100,
  "email_domain": "university.edu",
  "interests": ["AI", "Machine Learning"],
  "google_scholar_publication_ids": ["pub1", "pub2"],
  "google_scholar_id": "scholar123"
}
```

### Get All Researcher Cards
- **URL**: `/api/researcher_card/all`
- **Method**: `GET`
- **Response**:
```json
[
  {
    "id": 1,
    "name": "Researcher Name",
    "affiliation": "University Name",
    "citedby": 100,
    "email_domain": "university.edu",
    "interests": ["AI", "Machine Learning"],
    "google_scholar_publication_ids": ["pub1", "pub2"],
    "google_scholar_id": "scholar123"
  }
]
```

### Update Researcher Card
- **URL**: `/api/researcher_card/:id/update`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:
```json
{
  "name": "Updated Name",
  "affiliation": "Updated University",
  "citedby": 150,
  "email_domain": "university.edu",
  "interests": ["AI", "Machine Learning", "NLP"],
  "google_scholar_publication_ids": ["pub1", "pub2", "pub3"],
  "google_scholar_id": "scholar123"
}
```
- **Response**:
```json
{
  "id": 1,
  "name": "Updated Name",
  "affiliation": "Updated University",
  "citedby": 150,
  "email_domain": "university.edu",
  "interests": ["AI", "Machine Learning", "NLP"],
  "google_scholar_publication_ids": ["pub1", "pub2", "pub3"],
  "google_scholar_id": "scholar123"
}
```

## Researcher Card Thread Endpoints

### Get Researcher Card Threads
- **URL**: `/api/threads/researcher_card`
- **Method**: `GET`
- **Response**:
```json
[
  {
    "id": 1,
    "title": "Thread Title",
    "content": "Thread Content",
    "data": [],
    "researcher_id": 1
  }
]
```

### Get Researcher Card Thread
- **URL**: `/api/threads/researcher_card/:thread_id`
- **Method**: `GET`
- **Response**:
```json
{
  "id": 1,
  "title": "Thread Title",
  "content": "Thread Content",
  "replies": [
    {
      "id": "1234567890",
      "parent_id": null,
      "content": "Reply content"
    }
  ]
}
```

### Add Comment to Researcher Card Thread
- **URL**: `/api/comments/researcher_card`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Request Body**:
```json
{
  "thread_id": 1,
  "parent_id": "1234567890", // Optional, omit for top-level comments
  "content": "Comment content"
}
```
- **Response**: HTTP Status 201 (Created) on success

## Google Scholar Endpoints

### Get Google Scholar Information
- **URL**: `/api/google_scholar/:scholar_id`
- **Method**: `GET`
- **Response**:
```json
{
  "affiliation": "University Name",
  "citedby": 1000,
  "container_type": "type",
  "email_domain": "university.edu",
  "filled": ["field1", "field2"],
  "interests": ["AI", "Machine Learning"],
  "name": "Scholar Name",
  "organization": 12345,
  "publications": [
    {
      "author_pub_id": "pub_id",
      "bib": {
        "citation": "Citation text",
        "pub_year": "2023",
        "title": "Publication Title"
      },
      "citedby_url": "url",
      "cites_id": ["cite1", "cite2"],
      "container_type": "type",
      "filled": false,
      "num_citations": 50,
      "source": "source"
    }
  ],
  "scholar_id": "scholar123",
  "source": "source"
}
```

### Get Google Scholar Publication
- **URL**: `/api/google_scholar/:scholar_id/publication/:publication_id`
- **Method**: `GET`
- **Response**:
```json
{
  "author_pub_id": "pub_id",
  "bib": {
    "abstract": "Publication abstract",
    "author": "Author names",
    "citation": "Citation text",
    "journal": "Journal name",
    "number": "Number",
    "pages": "Pages",
    "pub_year": 2023,
    "publisher": "Publisher name",
    "title": "Publication title",
    "volume": "Volume"
  },
  "citedby_url": "url",
  "cites_id": ["cite1", "cite2"],
  "cites_per_year": {
    "2022": 10,
    "2023": 15
  }
}
```

### Update or Create Researcher from Google Scholar
- **URL**: `/api/google_scholar/update/:scholar_id`
- **Method**: `POST`
- **Response**:
```json
{
  "id": 1,
  "name": "Scholar Name",
  "affiliation": "University Name",
  "citedby": 1000,
  "email_domain": "university.edu",
  "interests": ["AI", "Machine Learning"],
  "google_scholar_publication_ids": ["pub1", "pub2"],
  "google_scholar_id": "scholar123"
}
```

## Data Structures

### Thread
```json
{
  "id": 1,
  "title": "Thread Title",
  "content": "Thread Content",
  "data": [], // Binary data
  "room_id": 123
}
```

### ResearcherCard
```json
{
  "id": 1,
  "name": "Researcher Name",
  "affiliation": "University Name",
  "citedby": 100,
  "email_domain": "university.edu",
  "interests": ["AI", "Machine Learning"],
  "google_scholar_publication_ids": ["pub1", "pub2"],
  "google_scholar_id": "scholar123"
}
```

### Reply/Comment
```json
{
  "id": "1234567890", // Snowflake ID as string
  "parent_id": "9876543210", // Optional, null for top-level comments
  "content": "Comment content"
}
```

### GoogleScholar
```json
{
  "affiliation": "University Name",
  "citedby": 1000,
  "container_type": "type",
  "email_domain": "university.edu",
  "filled": ["field1", "field2"],
  "interests": ["AI", "Machine Learning"],
  "name": "Scholar Name",
  "organization": 12345,
  "publications": [], // Array of PublicationNoFilled objects
  "scholar_id": "scholar123",
  "source": "source"
}
``` 